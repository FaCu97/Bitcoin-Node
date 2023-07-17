use std::{sync::{Arc, RwLock, mpsc::Sender}, net::TcpStream, path::Path, fs::File, io::Read, error::Error};

use chrono::{Utc, TimeZone};

use crate::{config::Config, logwriter::log_writer::{LogSender, write_in_log}, blocks::block_header::BlockHeader, custom_errors::NodeCustomErrors, messages::{headers_message::HeadersMessage, getheaders_message::GetHeadersMessage}};


// TODO: pasar 162003 como constante
// TODO: Documentar get_initial_headers
// TODO: Ver si se puede hacer solo una funcion para download_and_persist_headers y download_headers


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
        download_and_persist_headers(config, log_sender, headers, nodes)?;
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
fn download_and_persist_headers(
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
    while let Err(err) = download_and_persist_initial_headers_from_node(config, log_sender, &mut node, headers.clone(), &mut file) {
        write_in_log(
            &log_sender.error_log_sender,
            format!(
                "Fallo la descarga con el nodo --{:?}--, lo descarto y voy a intentar con otro. Error: {}",
                node.peer_addr(),
                err
            )
            .as_str(),
        );
        node = get_node_to_download_headers(nodes.clone())?;
    }
    // return node that donwloaded the header again to the vec of nodes
    nodes
        .write()
        .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
        .push(node);
    Ok(())
}

/// Descarga los headers de la blockchain desde los nodos conectados
/// En caso de que un nodo falle en la descarga, intenta con otro siempre y cuando tenga peers disponibles
/// Devuelve un error en caso de no poder descargar los headers desde nignun nodo peer
pub fn download_headers(
    config: &Arc<Config>,
    log_sender: &LogSender,
    nodes: Arc<RwLock<Vec<TcpStream>>>,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
    tx: Option<Sender<Vec<BlockHeader>>>,
) -> Result<(), NodeCustomErrors> {
    // get last node from list, if possible
    let mut node = get_node_to_download_headers(nodes.clone())?;
    while let Err(err) = download_missing_headers_from_node(config, log_sender, &mut node, headers.clone(), tx.clone()) {
        write_in_log(
            &log_sender.error_log_sender,
            format!(
                "Fallo la descarga con el nodo --{:?}--, lo descarto y voy a intentar con otro. Error: {}",
                node.peer_addr(),
                err
            )
            .as_str(),
        );
        if let NodeCustomErrors::ThreadChannelError(_) = err {
            return Err(NodeCustomErrors::ThreadChannelError("Error se cerro el channel que comunica la descarga de headers y bloques en paralelo".to_string()));
        }
        node = get_node_to_download_headers(nodes.clone())?;
    }
    // return node again to the list of nodes
    nodes
        .write()
        .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
        .push(node);
    /*
    let last_headers =
        compare_and_ask_for_last_headers(config, log_sender, nodes, headers_clone)?;
    if !last_headers.is_empty() {
        write_in_log(
            &log_sender.info_log_sender,
            format!(
                "Agrego ultimos {} headers enocontrados al comparar con todos los nodos",
                last_headers.len()
            )
            .as_str(),
        );
        tx.send(last_headers)
            .map_err(|err| NodeCustomErrors::ThreadChannelError(err.to_string()))?;
    }
    */
    Ok(())
}


/// Descarga los primeros headers (especificados en el archivo de configuracion) de la blockchain y los guarda en disco
/// Devuelve un error en caso de no poder descargar los headers exitosamente
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
        "{:?} headers descargados y guardados en disco",
        headers
            .read()
            .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
            .len()
    );
    Ok(())
}




/// Downloads the Block Headers from the received node and stores them in the received header list.
/// Starts sending them through the Block Download Channel when it finds the expected Block Header
/// If something fails, an error is returned
fn download_missing_headers_from_node(
    config: &Arc<Config>,
    log_sender: &LogSender,
    node: &mut TcpStream,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
    tx: Option<Sender<Vec<BlockHeader>>>,
) -> Result<(), NodeCustomErrors> {
    write_in_log(
        &log_sender.info_log_sender,
        format!(
            "Empiezo la descarga de todos los headers que faltan con nodo: {:?}\n",
            node.peer_addr()
        )
        .as_str(),
    );
    let mut multi_thread_download: bool = false;
    let sender: Sender<Vec<BlockHeader>>;
    if let Some(tx) = tx {
        sender = tx;
        multi_thread_download = true;
    };
    let mut first_block_found = false;
    request_headers_from_node(config, node, headers.clone())?;
    let mut headers_read= receive_headers_from_node(log_sender, node)?;
    // store headers in `global` vec `headers_guard`
    headers
        .write()
        .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
        .extend_from_slice(&headers_read);
    let mut first_headers_had_just_been_sent = false;
    while headers_read.len() == 2000 {
        request_headers_from_node(config, node, headers.clone())?;
        headers_read = receive_headers_from_node(log_sender, node)?;
        validate_headers(log_sender, &headers_read)?;
        if multi_thread_download {
            if headers
                .read()
                .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
                .len()
                == (config.height_first_block_to_download / 2000) * 2000
            {
                // Si no lo encuentra devuelve un error vacío, creo que esto está mal.
                let first_block_headers_to_download = search_first_header_block_to_download(
                    config,
                    headers_read.clone(),
                    &mut first_block_found,
                )
                .map_err(|err| NodeCustomErrors::FirstBlockNotFoundError(err.to_string()))?;
                write_in_log(
                    &log_sender.info_log_sender,
                    "Encontre primer bloque a descargar! Empieza descarga de bloques\n",
                );
                sender.send(first_block_headers_to_download)
                    .map_err(|err| NodeCustomErrors::ThreadChannelError(err.to_string()))?;
                first_headers_had_just_been_sent = true;
            }
            if first_block_found && !first_headers_had_just_been_sent {
                sender.send(headers_read.clone())
                    .map_err(|err| NodeCustomErrors::ThreadChannelError(err.to_string()))?;
            }
            first_headers_had_just_been_sent = false;
        }
        
        // store headers in `global` vec `headers_guard`
        headers
            .write()
            .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
            .extend_from_slice(&headers_read);
        println!(
            "{:?} headers descargados",
            headers
                .read()
                .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
                .len()
        );
    }
    Ok(())
}






















/*
***************************************************************************
************************ AUXILIAR FUNCTIONS *******************************
***************************************************************************
*/


/// Se fija por el ultimo header descargado y pide al nodo los headers siguientes con un mensaje getheaders
/// Devuelve un error en caso de no poder pedirlos correctamente
fn request_headers_from_node(
    config: &Arc<Config>,
    node: &mut TcpStream,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
) -> Result<(), NodeCustomErrors> {
    let last_hash_header_downloaded: [u8; 32] = if headers
        .read()
        .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
        .is_empty()
    {
        GENESIS_BLOCK
    } else {
        get_last_hash_header_downloaded(headers.clone())?
    };
    GetHeadersMessage::build_getheaders_message(config, vec![last_hash_header_downloaded])
        .write_to(node)
        .map_err(|err| NodeCustomErrors::WriteNodeError(err.to_string()))?;
    Ok(())
}


/// Recibe los headers del nodo y los guarda en disco
/// Devuelve un error en caso de no poder recibirlos correctamente
fn receive_and_persist_initial_headers_from_node(
    log_sender: &LogSender,
    node: &mut TcpStream,
    file: &mut File,
) -> Result<Vec<BlockHeader>, NodeCustomErrors> {
    let headers: Vec<BlockHeader> =
        HeadersMessage::read_from_node_and_write_to_file(log_sender, node, None, file)
            .map_err(|_| {
                NodeCustomErrors::BlockchainDownloadError("Error al leer y persistir headers iniciales".to_string())
            })?;
    Ok(headers)
}

/// Receives the header_message from the node.
/// Returns an array of BlockHeader or error if something fails.
pub fn receive_headers_from_node(
    log_sender: &LogSender,
    node: &mut TcpStream,
) -> Result<Vec<BlockHeader>, NodeCustomErrors> {
    let headers: Vec<BlockHeader> = HeadersMessage::read_from(log_sender, node, None)
        .map_err(|_| {
            NodeCustomErrors::BlockchainDownloadError("Error al leer headers".to_string())
        })?;
    Ok(headers)
}

/// Devuelve el hash del ultimo header descargado
fn get_last_hash_header_downloaded(headers: Arc<RwLock<Vec<BlockHeader>>>) -> Result<[u8;32], NodeCustomErrors> {
    let binding = headers
        .read()
        .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?;
    let last_header = binding
        .last();
    match last_header {
        Some(header) => Ok(header.hash()),
        None => return Err(NodeCustomErrors::BlockchainDownloadError("Error no hay headers descargados!\n".to_string())),
    } 
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


/// Valida que el header tenga la proof of work correcta
/// Devuelve un error en caso de que no sea valido
fn validate_headers(
    log_sender: &LogSender,
    headers: &Vec<BlockHeader>,
) -> Result<(), NodeCustomErrors> {
    for header in headers {
        if !header.validate() {
            write_in_log(
                &log_sender.error_log_sender,
                "Error en validacion de la proof of work de header",
            );
            return Err(NodeCustomErrors::InvalidHeaderError(
                "partial validation of header is invalid!".to_string(),
            ));
        }
    }
    Ok(())
}



/// Searches for the block headers that matches the defined timestamp defined by config.
/// If it is found, returns them and set the boolean to true.
/// In case of error it returns it.
fn search_first_header_block_to_download(
    config: &Arc<Config>,
    headers: Vec<BlockHeader>,
    found: &mut bool,
) -> Result<Vec<BlockHeader>, Box<dyn Error>> {
    let fecha_hora = Utc.datetime_from_str(
        &config.fecha_inicio_proyecto,
        &config.formato_fecha_inicio_proyecto,
    )?;
    let timestamp = fecha_hora.timestamp() as u32;

    let mut first_headers_from_blocks_to_download = vec![];
    for header in headers {
        if !(*found) && header.time > timestamp {
            *found = true;
        }
        if *found {
            first_headers_from_blocks_to_download.push(header);
        }
    }
    Ok(first_headers_from_blocks_to_download)
}