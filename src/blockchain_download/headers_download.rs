use std::{sync::{Arc, RwLock}, net::TcpStream, path::Path, fs::File, io::Read};

use crate::{config::Config, logwriter::log_writer::{LogSender, write_in_log}, blocks::block_header::BlockHeader, custom_errors::NodeCustomErrors, messages::{headers_message::HeadersMessage, getheaders_message::GetHeadersMessage}};


// TODO: pasar 162003 como constante
// TODO: Documentar get_initial_headers


const GENESIS_BLOCK: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0x09, 0x33, 0xea, 0x01, 0xad, 0x0e, 0xe9, 0x84, 0x20, 0x97, 0x79, 0xba,
    0xae, 0xc3, 0xce, 0xd9, 0x0f, 0xa3, 0xf4, 0x08, 0x71, 0x95, 0x26, 0xf8, 0xd7, 0x7f, 0x49, 0x43,
];


pub fn get_initial_headers(
    config: &Arc<Config>,
    log_sender: &LogSender,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
    nodes: Arc<RwLock<Vec<TcpStream>>>,
) -> Result<(), NodeCustomErrors> {
    if Path::new(&config.archivo_headers).exists() {
        match read_headers_from_disk(config, log_sender, headers) {
            Ok(_) => {
                write_in_log(
                    &log_sender.info_log_sender,
                    "Se leyeron correctamente los headers de disco",
                );
            }
            Err(err) => {
                write_in_log(
                    &log_sender.error_log_sender,
                    format!("No se pudieron leer los headers de disco. Error: {:?}", err).as_str(),
                );
                return Err(err)
            }
        }
    } else {
        download_first_headers(config, log_sender, headers, nodes)?;
    }
    Ok(())
}


/// Lee los headers de disco y los guarda en el vector de headers
/// Devuelve un error en caso de no poder leer el archivo
fn read_headers_from_disk(
    config: &Arc<Config>,
    log_sender: &LogSender,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
) -> Result<(), NodeCustomErrors> {
    write_in_log(
        &log_sender.info_log_sender,
        format!("Empiezo lectura de los primeros {} headers de disco", config.headers_in_disk).as_str()
    );
    let mut data: Vec<u8> = Vec::new();
    let mut file = File::open(&config.archivo_headers)
        .map_err(|err| NodeCustomErrors::OpeningFileError(err.to_string()))?;
    file.read_to_end(&mut data)
        .map_err(|err| NodeCustomErrors::ReadingFileError(err.to_string()))?;
    let mut amount = 0;
    let mut i = 0;
    while i < data.len() {
        amount += 2000;
        let mut message_bytes = Vec::new();
        message_bytes.extend_from_slice(&data[i..i + 162003]);
        let unmarshalled_headers = HeadersMessage::unmarshalling(&message_bytes).map_err(|err| NodeCustomErrors::UnmarshallingError(err.to_string()))?;
        headers
            .write()
            .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
            .extend_from_slice(&unmarshalled_headers);
        println!("{:?} headers leidos", amount);
        i += 162003;
    }
    Ok(())
}


/// Descarga los primeros headers de la blockchain, crea el archivo para guardarlos y los guarda en disco
/// En caso de que un nodo falle en la descarga, intenta con otro siempre y cuando tenga peers disponibles
/// Devuelve un error en caso de no poder descargar los headers desde nignun nodo peer
fn download_first_headers(
    config: &Arc<Config>,
    log_sender: &LogSender,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
    nodes: Arc<RwLock<Vec<TcpStream>>>,
) -> Result<(), NodeCustomErrors> {
    write_in_log(
        &log_sender.info_log_sender,
        format!("Empiezo descarga de los primeros {} headers para guardarlos en disco", config.headers_in_disk).as_str(),
    );
    let mut file = File::create(&config.archivo_headers)
        .map_err(|err| NodeCustomErrors::OpeningFileError(err.to_string()))?;
    // get last node from list, if possible
    let mut node = get_node_to_download_headers(nodes.clone())?;
    let mut node_ip = node.peer_addr().map_err(|err| NodeCustomErrors::SocketError(err.to_string()))?;
    // first try to download headers from node
    let mut download_result =
        download_and_persist_initial_headers_from_node(config, log_sender, &mut node, headers.clone(), &mut file);
    while let Err(err) = download_result {
        write_in_log(
            &log_sender.error_log_sender,
            format!(
                "Fallo la descarga con el nodo --{:?}--, lo descarto y voy a intentar con otro. Error: {}",
                node_ip,
                err
            )
            .as_str(),
        );
        if let NodeCustomErrors::ThreadChannelError(_) = err {
            return Err(NodeCustomErrors::ThreadChannelError("Error se cerro el channel que comunica la descarga de headers y bloques en paralelo".to_string()));
        }
        node = get_node_to_download_headers(nodes.clone())?;
        node_ip = node.peer_addr().map_err(|err| NodeCustomErrors::SocketError(err.to_string()))?;
        // try to download headers from that node
        download_result = download_and_persist_initial_headers_from_node(
            config,
            log_sender,
            &mut node,
            headers.clone(),
            &mut file,
        );
    }
    // return node again to the list of nodes
    nodes
        .write()
        .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
        .push(node);
    Ok(())
}





/// Downloads the first headers (specified in configuration file) from the node.
/// Returns an error if something fails
fn download_and_persist_initial_headers_from_node(
    config: &Arc<Config>,
    log_sender: &LogSender,
    node: &mut TcpStream,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
    file: &mut File,
) -> Result<(), NodeCustomErrors> {
    write_in_log(
        &log_sender.info_log_sender,
        format!(
            "Empiezo descarga de headers con nodo: {:?}\n",
            node.peer_addr()
        )
        .as_str(),
    );
    while headers
        .read()
        .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
        .len()
        < config.headers_in_disk
    {
        println!(
            "{:?}",
            headers
                .read()
                .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
                .len()
        );
        request_headers_from_node(config, node, headers.clone())?;
        let headers_read = receive_and_persist_initial_headers_from_node(log_sender, node, file)?;
        // store headers in `global` vec `headers_guard`
        headers
            .write()
            .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
            .extend_from_slice(&headers_read);
    }

    println!(
        "{:?} headers descargados",
        headers
            .read()
            .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
            .len()
    );
    Ok(())
}


/// Checks the last header available on the local chain and requests the followings from the received node.
/// Returns error if it fails, otherwise return the node.
fn request_headers_from_node(
    config: &Arc<Config>,
    node: &mut TcpStream,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
) -> Result<(), NodeCustomErrors> {
    // write first getheaders message with genesis block
    let last_hash_header_downloaded: [u8; 32] = if headers
        .read()
        .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
        .is_empty()
    {
        GENESIS_BLOCK
    } else {
        headers
            .read()
            .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
            .last()
            .ok_or("No se pudo obtener el último elemento del vector de 2000 headers")
            .map_err(|err| NodeCustomErrors::CanNotRead(err.to_string()))?
            .hash()
    };
    GetHeadersMessage::build_getheaders_message(config, vec![last_hash_header_downloaded])
        .write_to(node)
        .map_err(|err| NodeCustomErrors::WriteNodeError(err.to_string()))?;
    Ok(())
}



/// Receives from the node and write to a file the headers
fn receive_and_persist_initial_headers_from_node(
    log_sender: &LogSender,
    node: &mut TcpStream,
    file: &mut File,
) -> Result<Vec<BlockHeader>, NodeCustomErrors> {
    // read first 2000 headers from headers message answered from node
    let headers: Vec<BlockHeader> =
        HeadersMessage::read_from_node_and_write_to_file(log_sender, node, None, file)
            .map_err(|_| {
                NodeCustomErrors::BlockchainDownloadError("Error al leer primeros 2000 headers".to_string())
            })?;
    Ok(headers)
}

/// Receives the header_message from the node.
/// Returns an array of BlockHeader or error if something fails.
fn receive_headers_from_node(
    log_sender: &LogSender,
    node: &mut TcpStream,
) -> Result<Vec<BlockHeader>, NodeCustomErrors> {
    // read first 2000 headers from headers message answered from node
    let headers: Vec<BlockHeader> = HeadersMessage::read_from(log_sender, node, None)
        .map_err(|_| {
            NodeCustomErrors::ReadNodeError("Error al leer primeros 2000 headers".to_string())
        })?;
    Ok(headers)
}




/// Devuelve el ultimo nodo de la lista de nodos conectados para descargar los headers de la blockchain
/// En caso de no haber mas nodos disponibles devuelve un error
fn get_node_to_download_headers(nodes: Arc<RwLock<Vec<TcpStream>>>) -> Result<TcpStream, NodeCustomErrors> {
    let node = nodes
        .write()
        .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
        .pop();
    match node {
        Some(node) => Ok(node),
        None => return Err(NodeCustomErrors::BlockchainDownloadError("Error no hay mas nodos conectados para descargar los headers de la blockchain!\n".to_string())),
    }
    
}