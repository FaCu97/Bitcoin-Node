use crate::blocks::block::Block;
use crate::blocks::block_header::BlockHeader;
use crate::config::Config;
use crate::logwriter::log_writer::{write_in_log, LogSender};
use crate::messages::{
    block_message::BlockMessage, get_data_message::GetDataMessage,
    getheaders_message::GetHeadersMessage, headers_message::HeadersMessage, inventory::Inventory,
};
use chrono::{TimeZone, Utc};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::net::TcpStream;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::{fmt, thread, vec};

// todo: Pasar constantes a config
// todo: Agregar validacion de headers

const GENESIS_BLOCK: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0x09, 0x33, 0xea, 0x01, 0xad, 0x0e, 0xe9, 0x84, 0x20, 0x97, 0x79, 0xba,
    0xae, 0xc3, 0xce, 0xd9, 0x0f, 0xa3, 0xf4, 0x08, 0x71, 0x95, 0x26, 0xf8, 0xd7, 0x7f, 0x49, 0x43,
];

type DownloadResult = Result<(), DownloadError>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DownloadError {
    ThreadJoinError(String),
    LockError(String),
    ReadNodeError(String),
    WriteNodeError(String),
    CanNotRead(String),
    ThreadChannelError(String),
    FirstBlockNotFoundError(String),
    InvalidHeaderError(String),
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DownloadError::ThreadJoinError(msg) => write!(f, "ThreadJoinError Error: {}", msg),
            DownloadError::LockError(msg) => write!(f, "LockError Error: {}", msg),
            DownloadError::ReadNodeError(msg) => {
                write!(f, "Can not read from socket Error: {}", msg)
            }
            DownloadError::WriteNodeError(msg) => {
                write!(f, "Can not write in socket Error: {}", msg)
            }
            DownloadError::CanNotRead(msg) => write!(f, "No more elements in list Error: {}", msg),
            DownloadError::ThreadChannelError(msg) => {
                write!(f, "Can not send elements to channel Error: {}", msg)
            }
            DownloadError::FirstBlockNotFoundError(msg) => {
                write!(f, "First block to download not found Error: {}", msg)
            }
            DownloadError::InvalidHeaderError(msg) => write!(f, "Not valid header Error: {}", msg),
        }
    }
}

impl Error for DownloadError {}

// cambiar esta constante y borrar el archivo first_headers.csv (si existe) para ver cambios
const CANTIDAD_HEADERS_EN_DISCO: usize = 2300000;
const ALTURA_PRIMER_BLOQUE_A_DESCARGAR: usize = 2428000;
const ALTURA_PRIMER_BLOQUE: usize = 2428246;
type HeadersBlocksTuple = (
    Arc<RwLock<Vec<BlockHeader>>>,
    Arc<RwLock<HashMap<[u8; 32], Block>>>,
);

/// Searches for the block headers that matches the defined timestamp defined by config.
/// If it is found, returns them and set the boolean to true.
/// In case of error it returns it.
fn search_first_header_block_to_download(
    config: Arc<Config>,
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

/// Validates all the BlockHeaders received.
/// Returns an error if the validation fails.
fn validate_headers(log_sender: LogSender, headers: Vec<BlockHeader>) -> Result<(), DownloadError> {
    for header in headers {
        if !header.validate() {
            write_in_log(
                &log_sender.error_log_sender,
                "Error en validacion de la proof of work de header",
            );
            return Err(DownloadError::InvalidHeaderError(
                "partial validation of header is invalid!".to_string(),
            ));
        }
    }
    Ok(())
}

/// Downloads the Block Headers from the received node and stores them in the received header list.
/// Starts sending them through the Block Download Channel when it finds the expected Block Header
/// If something fails, an error is returned
fn download_headers_from_node(
    config: Arc<Config>,
    log_sender: LogSender,
    mut node: TcpStream,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
    tx: Sender<Vec<BlockHeader>>,
) -> Result<TcpStream, DownloadError> {
    write_in_log(
        &log_sender.info_log_sender,
        format!(
            "Empiezo descarga de headers con nodo: {:?}\n",
            node.peer_addr()
        )
        .as_str(),
    );

    let mut first_block_found = false;
    node = request_headers_from_node(config.clone(), node, headers.clone())?;
    let mut headers_read: Vec<BlockHeader>;
    (node, headers_read) = receive_headers_from_node(log_sender.clone(), node)?;

    // store headers in `global` vec `headers_guard`
    headers
        .write()
        .map_err(|err| DownloadError::LockError(err.to_string()))?
        .extend_from_slice(&headers_read);
    let mut first_headers_had_just_been_sent = false;
    while headers_read.len() == 2000 {
        node = request_headers_from_node(config.clone(), node, headers.clone())?;
        (node, headers_read) = receive_headers_from_node(log_sender.clone(), node)?;

        validate_headers(log_sender.clone(), headers_read.clone())?;
        if headers
            .read()
            .map_err(|err| DownloadError::LockError(err.to_string()))?
            .len()
            == ALTURA_PRIMER_BLOQUE_A_DESCARGAR
        {
            // Si no lo encuentra devuelve un error vacío, creo que esto está mal.
            let first_block_headers_to_download = search_first_header_block_to_download(
                config.clone(),
                headers_read.clone(),
                &mut first_block_found,
            )
            .map_err(|err| DownloadError::FirstBlockNotFoundError(err.to_string()))?;
            write_in_log(
                &log_sender.info_log_sender,
                "Encontre primer bloque a descargar! Empieza descarga de bloques\n",
            );
            tx.send(first_block_headers_to_download)
                .map_err(|err| DownloadError::ThreadChannelError(err.to_string()))?;
            first_headers_had_just_been_sent = true;
        }
        if first_block_found && !first_headers_had_just_been_sent {
            tx.send(headers_read.clone())
                .map_err(|err| DownloadError::ThreadChannelError(err.to_string()))?;
        }
        first_headers_had_just_been_sent = false;
        // store headers in `global` vec `headers_guard`
        headers
            .write()
            .map_err(|err| DownloadError::LockError(err.to_string()))?
            .extend_from_slice(&headers_read);
        println!(
            "{:?} headers descargados",
            headers
                .read()
                .map_err(|err| DownloadError::LockError(err.to_string()))?
                .len()
        );
    }
    Ok(node)
}

/// Checks the last header available on the local chain and requests the followings from the received node.
/// Returns error if it fails, otherwise return the node.
fn request_headers_from_node(
    config: Arc<Config>,
    mut node: TcpStream,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
) -> Result<TcpStream, DownloadError> {
    // write first getheaders message with genesis block
    let last_hash_header_downloaded: [u8; 32] = if headers
        .read()
        .map_err(|err| DownloadError::LockError(err.to_string()))?
        .is_empty()
    {
        GENESIS_BLOCK
    } else {
        headers
            .read()
            .map_err(|err| DownloadError::LockError(err.to_string()))?
            .last()
            .ok_or("No se pudo obtener el último elemento del vector de 2000 headers")
            .map_err(|err| DownloadError::CanNotRead(err.to_string()))?
            .hash()
    };
    GetHeadersMessage::build_getheaders_message(config, vec![last_hash_header_downloaded])
        .write_to(&mut node)
        .map_err(|err| DownloadError::WriteNodeError(err.to_string()))?;
    Ok(node)
}

/// Receives the header_message from the node.
/// Returns an array of BlockHeader or error if something fails.
fn receive_headers_from_node(
    log_sender: LogSender,
    mut node: TcpStream,
) -> Result<(TcpStream, Vec<BlockHeader>), DownloadError> {
    // read first 2000 headers from headers message answered from node
    let headers: Vec<BlockHeader> = HeadersMessage::read_from(log_sender, &mut node, None)
        .map_err(|_| {
            DownloadError::ReadNodeError("error al leer primeros 2000 headers".to_string())
        })?;
    Ok((node, headers))
}

/// Download the headers from a node of the list.
/// If it fails, the node is discarded and try to download from another node.
/// If all the nodes fail, retuns an error.
fn download_headers(
    config: Arc<Config>,
    log_sender: LogSender,
    nodes: Arc<RwLock<Vec<TcpStream>>>,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
    tx: Sender<Vec<BlockHeader>>,
) -> DownloadResult {
    // get last node from list, if possible
    let mut node = nodes
        .write()
        .map_err(|err| DownloadError::LockError(err.to_string()))?
        .pop()
        .ok_or("Error no hay mas nodos para descargar los headers!\n")
        .map_err(|err| DownloadError::CanNotRead(err.to_string()))?;
    let headers_clone = headers.clone();
    let tx_clone = tx.clone();
    // first try to dowload headers from node
    let mut download_result =
        download_headers_from_node(config.clone(), log_sender.clone(), node, headers, tx);
    while let Err(err) = download_result {
        write_in_log(
            &log_sender.error_log_sender,
            format!(
                "Fallo la descarga con el nodo, lo descarto y voy a intentar con otro. Error: {}",
                err
            )
            .as_str(),
        );
        if let DownloadError::ThreadChannelError(_) = err {
            return Err(DownloadError::ThreadChannelError("Error se cerro el channel que comunica la descarga de headers y bloques en paralelo".to_string()));
        }
        node = nodes
            .write()
            .map_err(|err| DownloadError::LockError(err.to_string()))?
            .pop()
            .ok_or("Error no hay mas nodos para descargar los headers! Todos fallaron \n")
            .map_err(|err| DownloadError::CanNotRead(err.to_string()))?;
        // try to download headers from that node
        download_result = download_headers_from_node(
            config.clone(),
            log_sender.clone(),
            node,
            headers_clone.clone(),
            tx_clone.clone(),
        );
    }
    // get the node which download the headers correctly
    node = download_result.map_err(|_| {
        DownloadError::ReadNodeError(
            "Descarga fallida con todos los nodos conectados! \n".to_string(),
        )
    })?;
    // return node again to the list of nodes
    nodes
        .write()
        .map_err(|err| DownloadError::LockError(err.to_string()))?
        .push(node);
    /*
    let last_headers =
        compare_and_ask_for_last_headers(config, log_sender.clone(), nodes, headers_clone)?;
    if !last_headers.is_empty() {
        write_in_log(
            log_sender.info_log_sender,
            format!(
                "Agrego ultimos {} headers enocontrados al comparar con todos los nodos",
                last_headers.len()
            )
            .as_str(),
        );
        tx.send(last_headers)
            .map_err(|err| DownloadError::ThreadChannelError(err.to_string()))?;
    }
    */
    Ok(())
}

/// # Descarga de bloques
/// Realiza la descarga de bloques de forma concurrente.
/// ### Recibe:
/// - La referencia a la lista de nodos a los que se conectar.
/// - La referencia a la lista de bloques donde los almacenará
/// - La referencia a los block headers descargados
/// - El channel por donde recibe los block headers
/// - El channel por donde devuelve los block headers cuando no los puede descargar
///
/// ### Manejo de errores:
/// Vuelve a intentar la descarga con un nuevo nodo, en los siguientes casos:
/// - No se pudo realizar la solicitud de los bloques
/// - No se pudo recibir el bloque
///
/// ### Devuelve:
/// - Ok o un error si no se puede completar la descarga
fn download_blocks(
    config: Arc<Config>,
    log_sender: LogSender,
    nodes: Arc<RwLock<Vec<TcpStream>>>,
    blocks: Arc<RwLock<HashMap<[u8; 32], Block>>>,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
    rx: Receiver<Vec<BlockHeader>>,
    tx: Sender<Vec<BlockHeader>>,
) -> Result<(), DownloadError> {
    // recieves in the channel the vec of headers sent by the function downloading headers
    for received in rx {
        // acá recibo 2000 block headers
        let mut n_threads = config.n_threads;
        if received.len() < config.blocks_download_per_node {
            n_threads = 1;
        }
        if received.is_empty() {
            return Err(DownloadError::ThreadChannelError(
                "Se recibio una lista con 0 elementos!".to_string(),
            ));
        }
        let chunk_size = (received.len() as f64 / n_threads as f64).ceil() as usize;
        // divides the vec into 8 with the same lenght (or same lenght but the last with less)
        let blocks_headers_chunks = Arc::new(RwLock::new(
            received
                .chunks(chunk_size)
                .map(|chunk| chunk.to_vec())
                .collect::<Vec<_>>(),
        ));
        let mut handle_join = vec![];

        for i in 0..n_threads {
            // este ciclo crea la cantidad de threads simultaneos
            let config_cloned = config.clone();
            let tx_cloned = tx.clone();
            let nodes_pointer_clone = nodes.clone();
            let block_headers_chunk_clone = Arc::clone(&blocks_headers_chunks);
            let blocks_pointer_clone = Arc::clone(&blocks);
            let node = nodes_pointer_clone
                .write()
                .map_err(|err| DownloadError::LockError(err.to_string()))?
                .pop()
                .ok_or("Error no hay mas nodos para descargar los bloques!\n")
                .map_err(|err| DownloadError::CanNotRead(err.to_string()))?;

            if i >= block_headers_chunk_clone
                .read()
                .map_err(|err| DownloadError::LockError(err.to_string()))?
                .len()
            {
                // Este caso evita acceder a una posición fuera de rango
                // Significa que no hay más chunks con bloques para descargar
                break;
            }

            let block_headers = block_headers_chunk_clone
                .write()
                .map_err(|err| DownloadError::LockError(err.to_string()))?[i]
                .clone();
            let log_sender_clone = log_sender.clone();
            handle_join.push(thread::spawn(move || {
                download_blocks_single_thread(
                    config_cloned,
                    log_sender_clone,
                    block_headers,
                    node,
                    tx_cloned,
                    blocks_pointer_clone,
                    nodes_pointer_clone,
                )
            }));
        }
        for h in handle_join {
            h.join()
                .map_err(|err| DownloadError::ThreadJoinError(format!("{:?}", err)))??;
        }
        let bloques_descargados = blocks
            .read()
            .map_err(|err| DownloadError::LockError(err.to_string()))?
            .len();
        let cantidad_headers_descargados = headers
            .read()
            .map_err(|err| DownloadError::LockError(err.to_string()))?
            .len();
        let bloques_a_descargar = cantidad_headers_descargados - ALTURA_PRIMER_BLOQUE + 1;
        if bloques_descargados == bloques_a_descargar {
            write_in_log(&log_sender.info_log_sender, format!("Se terminaron de descargar todos los bloques correctamente! BLOQUES DESCARGADOS: {}\n", bloques_descargados).as_str());
            return Ok(());
        }
    }
    Ok(())
}

/// Downloads all the blocks from the same node, in the same thread.
/// The blocks are stored in the blocks list received by parameter.
/// In the end, the node is also return to the list of nodes
/// ## Errors
/// In case of Read or Write error on the node, the function is terminated, discarding the problematic node.
/// The downloaded blocks upon the error are discarded, so the whole block chunk can be downloaded again from another node
/// In other cases, it returns error.
fn download_blocks_single_thread(
    config: Arc<Config>,
    log_sender: LogSender,
    block_headers: Vec<BlockHeader>,
    mut node: TcpStream,
    tx: Sender<Vec<BlockHeader>>,
    blocks_pointer_clone: Arc<RwLock<HashMap<[u8; 32], Block>>>,
    nodes_pointer_clone: Arc<RwLock<Vec<TcpStream>>>,
) -> DownloadResult {
    let mut current_blocks: HashMap<[u8; 32], Block> = HashMap::new();
    // el thread recibe 250 bloques
    let block_headers_thread = block_headers.clone();
    write_in_log(
        &log_sender.info_log_sender,
        format!(
            "Voy a descargar {:?} bloques del nodo {:?}",
            block_headers.len(),
            node.peer_addr()
        )
        .as_str(),
    );
    for chunk_llamada in block_headers.chunks(config.blocks_download_per_node) {
        match request_blocks_from_node(
            log_sender.clone(),
            &node,
            chunk_llamada,
            &block_headers_thread,
            tx.clone(),
        ) {
            Ok(_) => {}
            Err(DownloadError::WriteNodeError(_)) => return Ok(()),
            Err(error) => return Err(error),
        }
        let received_blocks;
        (node, received_blocks) = match receive_requested_blocks_from_node(
            log_sender.clone(),
            node,
            chunk_llamada,
            &block_headers_thread,
            tx.clone(),
        ) {
            Ok(blocks) => blocks,
            Err(DownloadError::ReadNodeError(_)) => return Ok(()),
            Err(error) => return Err(error),
        };

        for block in received_blocks.into_iter() {
            current_blocks.insert(block.hash(), block);
        }
    }
    blocks_pointer_clone
        .write()
        .map_err(|err| DownloadError::LockError(err.to_string()))?
        .extend(current_blocks);
    write_in_log(
        &log_sender.info_log_sender,
        format!(
            "BLOQUES DESCARGADOS: {:?}",
            blocks_pointer_clone
                .read()
                .map_err(|err| DownloadError::LockError(err.to_string()))?
                .len()
        )
        .as_str(),
    );
    println!(
        "{:?} bloques descargados",
        blocks_pointer_clone
            .read()
            .map_err(|err| DownloadError::LockError(err.to_string()))?
            .len()
    );
    nodes_pointer_clone
        .write()
        .map_err(|err| DownloadError::LockError(err.to_string()))?
        .push(node);
    Ok(())
}

/// Requests the blocks to the node.
/// ## Errors
/// In case of error while sending the message, it returns the block headers back to the channel so
/// they can be downloaded from another node. If this cannot be done, returns an error.
fn request_blocks_from_node(
    log_sender: LogSender,
    mut node: &TcpStream,
    chunk_llamada: &[BlockHeader],
    block_headers_thread: &[BlockHeader],
    tx: Sender<Vec<BlockHeader>>,
) -> DownloadResult {
    //  Acá ya separé los 250 en chunks de 16 para las llamadas
    let mut inventory = vec![];
    for block in chunk_llamada {
        inventory.push(Inventory::new_block(block.hash()));
    }
    match GetDataMessage::new(inventory).write_to(&mut node) {
        Ok(_) => Ok(()),
        Err(err) => {
            write_in_log(&log_sender.error_log_sender,format!("Error: No puedo pedir {:?} cantidad de bloques del nodo: {:?}. Se los voy a pedir a otro nodo", chunk_llamada.len(), node.peer_addr()).as_str());

            tx.send(block_headers_thread.to_vec())
                .map_err(|err| DownloadError::ThreadChannelError(err.to_string()))?;
            // falló el envio del mensaje, tengo que intentar con otro nodo
            // si hago return, termino el thread.
            // tengo que enviar todos los bloques que tenía ese thread
            Err(DownloadError::WriteNodeError(format!("{:?}", err)))
        }
    }
}

/// Receives the blocks previously requested to the node.
/// Returns an array with the blocks.
/// In case of error while receiving the message, it returns the block headers back to the channel so
/// they can be downloaded from another node. If this cannot be done, returns an error.
fn receive_requested_blocks_from_node(
    log_sender: LogSender,
    mut node: TcpStream,
    chunk_llamada: &[BlockHeader],
    block_headers_thread: &[BlockHeader],
    tx: Sender<Vec<BlockHeader>>,
) -> Result<(TcpStream, Vec<Block>), DownloadError> {
    // Acá tengo que recibir los 16 bloques (o menos) de la llamada
    let mut current_blocks: Vec<Block> = Vec::new();
    for _ in 0..chunk_llamada.len() {
        let bloque = match BlockMessage::read_from(log_sender.clone(), &mut node) {
            Ok(bloque) => bloque,
            Err(err) => {
                write_in_log(&log_sender.error_log_sender,format!("No puedo descargar {:?} de bloques del nodo: {:?}. Se los voy a pedir a otro nodo y descarto este. Error: {err}", chunk_llamada.len(), node.peer_addr()).as_str());
                tx.send(block_headers_thread.to_vec())
                    .map_err(|err| DownloadError::ThreadChannelError(err.to_string()))?;
                // falló la recepción del mensaje, tengo que intentar con otro nodo
                // termino el nodo con el return
                return Err(DownloadError::ReadNodeError(format!(
                    "Error al recibir el mensaje `block`: {:?}",
                    err
                )));
            }
        };
        let validation_result = bloque.validate();
        if !validation_result.0 {
            write_in_log(&log_sender.error_log_sender,format!("El bloque no pasó la validación. {:?}. Se los voy a pedir a otro nodo y descarto este.", validation_result.1).as_str());
            tx.send(block_headers_thread.to_vec())
                .map_err(|err| DownloadError::ThreadChannelError(err.to_string()))?;
            return Err(DownloadError::ReadNodeError(format!(
                "Error al recibir el mensaje `block`: {:?}",
                validation_result.1
            )));
        }
        //bloque.set_utxos(); // seteo utxos de las transacciones del bloque
        current_blocks.push(bloque);
    }
    Ok((node, current_blocks))
}

/// Recieves a list of TcpStreams that are the connection with nodes already established and downloads
/// all the headers from the blockchain and the blocks from a config date. Returns the headers and blocks in
/// two separete lists in case of exit or an error in case of faliure
pub fn initial_block_download(
    config: Arc<Config>,
    log_sender: LogSender,
    nodes: Arc<RwLock<Vec<TcpStream>>>,
) -> Result<HeadersBlocksTuple, DownloadError> {
    write_in_log(
        &log_sender.info_log_sender,
        "EMPIEZA DESCARGA INICIAL DE BLOQUES",
    );
    let config_cloned = config.clone();
    let headers = vec![];
    let pointer_to_headers = Arc::new(RwLock::new(headers));
    get_initial_headers(
        config.clone(),
        log_sender.clone(),
        pointer_to_headers.clone(),
        nodes.clone(),
    )
    .map_err(|err| DownloadError::ThreadJoinError(format!("{:?}", err)))?;

    let blocks: HashMap<[u8; 32], Block> = HashMap::new();
    let pointer_to_blocks = Arc::new(RwLock::new(blocks));
    // channel to comunicate headers download thread with blocks download thread
    let (tx, rx) = channel();
    let tx_cloned = tx.clone();
    let (pointer_to_headers_clone, pointer_to_nodes_clone) =
        (Arc::clone(&pointer_to_headers), Arc::clone(&nodes));
    let log_sender_clone = log_sender.clone();
    let headers_thread = thread::spawn(move || {
        download_headers(
            config_cloned,
            log_sender_clone.clone(),
            pointer_to_nodes_clone,
            pointer_to_headers_clone,
            tx,
        )
    });
    let pointer_to_headers_clone_for_blocks = Arc::clone(&pointer_to_headers);
    let pointer_to_blocks_clone = Arc::clone(&pointer_to_blocks);
    let log_sender_clone = log_sender.clone();
    let blocks_thread = thread::spawn(move || {
        download_blocks(
            config.clone(),
            log_sender_clone,
            nodes,
            pointer_to_blocks_clone,
            pointer_to_headers_clone_for_blocks,
            rx,
            tx_cloned,
        )
    });
    headers_thread
        .join()
        .map_err(|err| DownloadError::ThreadJoinError(format!("{:?}", err)))??;
    blocks_thread
        .join()
        .map_err(|err| DownloadError::ThreadJoinError(format!("{:?}", err)))??;
    let headers = &*pointer_to_headers
        .read()
        .map_err(|err| DownloadError::LockError(format!("{:?}", err)))?;
    let blocks = &*pointer_to_blocks
        .read()
        .map_err(|err| DownloadError::LockError(format!("{:?}", err)))?;
    write_in_log(
        &log_sender.info_log_sender,
        format!("TOTAL DE HEADERS DESCARGADOS: {}", headers.len()).as_str(),
    );
    write_in_log(
        &log_sender.info_log_sender,
        format!("TOTAL DE BLOQUES DESCARGADOS: {}\n", blocks.len()).as_str(),
    );

    Ok((pointer_to_headers.clone(), pointer_to_blocks.clone()))
}

/// Gets the first 2.3 million headers.
/// It reads them from disk. If the file doesn't exist, it is also downloaded.
/// Returns error if something fails.
fn get_initial_headers(
    config: Arc<Config>,
    log_sender: LogSender,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
    nodes: Arc<RwLock<Vec<TcpStream>>>,
) -> Result<(), Box<dyn Error>> {
    if Path::new(&config.archivo_headers).exists() {
        read_first_headers_from_disk(config, log_sender.clone(), headers).map_err(|err| {
            write_in_log(
                &log_sender.error_log_sender,
                format!("Error al descargar primeros 2 millones de headers de disco. {err}")
                    .as_str(),
            );
            DownloadError::CanNotRead(format!(
                "Error al leer primeros 2 millones de headers. {}",
                err
            ))
        })?;
    } else {
        download_first_headers(config, log_sender, headers, nodes)?;
    }
    Ok(())
}

/// Downloads the headers, stores them into the headers array and create the file and save them on disk.
/// If a node fails, the download is continud from another nodes.
fn download_first_headers(
    config: Arc<Config>,
    log_sender: LogSender,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
    nodes: Arc<RwLock<Vec<TcpStream>>>,
) -> DownloadResult {
    write_in_log(
        &log_sender.info_log_sender,
        "Empiezo descarga de los primeros headers para guardarlos en disco",
    );
    let mut file = File::create(&config.archivo_headers)
        .map_err(|err| DownloadError::CanNotRead(err.to_string()))?;
    // get last node from list, if possible
    let mut node = nodes
        .write()
        .map_err(|err| DownloadError::LockError(err.to_string()))?
        .pop()
        .ok_or("Error no hay mas nodos para descargar los headers!\n")
        .map_err(|err| DownloadError::CanNotRead(err.to_string()))?;
    let headers_clone = headers.clone();
    // first try to download headers from node
    let mut download_result = download_initial_headers_from_node(
        config.clone(),
        log_sender.clone(),
        node,
        headers,
        &mut file,
    );
    while let Err(err) = download_result {
        write_in_log(
            &log_sender.error_log_sender,
            format!(
                "Fallo la descarga con el nodo, lo descarto y voy a intentar con otro. Error: {}",
                err
            )
            .as_str(),
        );
        if let DownloadError::ThreadChannelError(_) = err {
            return Err(DownloadError::ThreadChannelError("Error se cerro el channel que comunica la descarga de headers y bloques en paralelo".to_string()));
        }
        node = nodes
            .write()
            .map_err(|err| DownloadError::LockError(err.to_string()))?
            .pop()
            .ok_or("Error no hay mas nodos para descargar los headers! Todos fallaron \n")
            .map_err(|err| DownloadError::CanNotRead(err.to_string()))?;
        // try to download headers from that node
        download_result = download_initial_headers_from_node(
            config.clone(),
            log_sender.clone(),
            node,
            headers_clone.clone(),
            &mut file,
        );
    }
    // get the node which download the headers correctly
    node = download_result.map_err(|_| {
        DownloadError::ReadNodeError(
            "Descarga fallida con todos los nodos conectados! \n".to_string(),
        )
    })?;
    // return node again to the list of nodes
    nodes
        .write()
        .map_err(|err| DownloadError::LockError(err.to_string()))?
        .push(node);

    Ok(())
}

/// Downloads the first 2.3 million headers from the node.
/// Returns an error if something fails
fn download_initial_headers_from_node(
    config: Arc<Config>,
    log_sender: LogSender,
    mut node: TcpStream,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
    file: &mut File,
) -> Result<TcpStream, DownloadError> {
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
        .map_err(|err| DownloadError::LockError(err.to_string()))?
        .len()
        < CANTIDAD_HEADERS_EN_DISCO
    {
        println!(
            "{:?}",
            headers
                .read()
                .map_err(|err| DownloadError::LockError(err.to_string()))?
                .len()
        );
        node = request_headers_from_node(config.clone(), node, headers.clone())?;
        let headers_read: Vec<BlockHeader>;
        (node, headers_read) = receive_initial_headers_from_node(log_sender.clone(), node, file)?;
        // store headers in `global` vec `headers_guard`
        headers
            .write()
            .map_err(|err| DownloadError::LockError(err.to_string()))?
            .extend_from_slice(&headers_read);
    }

    println!(
        "{:?} headers descargados",
        headers
            .read()
            .map_err(|err| DownloadError::LockError(err.to_string()))?
            .len()
    );
    Ok(node)
}

/// Receives from the node and write to a file the headers
fn receive_initial_headers_from_node(
    log_sender: LogSender,
    mut node: TcpStream,
    file: &mut File,
) -> Result<(TcpStream, Vec<BlockHeader>), DownloadError> {
    // read first 2000 headers from headers message answered from node
    let headers: Vec<BlockHeader> =
        HeadersMessage::read_from_node_and_write_to_file(log_sender, &mut node, None, file)
            .map_err(|_| {
                DownloadError::ReadNodeError("error al leer primeros 2000 headers".to_string())
            })?;
    Ok((node, headers))
}

/// Loads the headers from disk
fn read_first_headers_from_disk(
    config: Arc<Config>,
    log_sender: LogSender,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
) -> Result<(), Box<dyn Error>> {
    write_in_log(
        &log_sender.info_log_sender,
        "Empiezo lectura de los primeros headers de disco",
    );
    let mut data: Vec<u8> = Vec::new();
    let mut file = File::open(&config.archivo_headers)
        .map_err(|err| DownloadError::CanNotRead(err.to_string()))?;
    file.read_to_end(&mut data)
        .map_err(|err| DownloadError::CanNotRead(err.to_string()))?;
    let mut cant = 0;
    let mut i = 0;
    while i < data.len() {
        cant += 2000;
        let mut message_bytes = Vec::new();
        message_bytes.extend_from_slice(&data[i..i + 162003]);
        let unmarshalled_headers = HeadersMessage::unmarshalling(&message_bytes)?;
        headers
            .write()
            .map_err(|err| DownloadError::LockError(err.to_string()))?
            .extend_from_slice(&unmarshalled_headers);
        println!("{:?} headers leidos", cant);
        i += 162003;
    }
    write_in_log(
        &log_sender.info_log_sender,
        "Termino lectura de los primeros headers de disco",
    );

    Ok(())
}

/*
/// Once the headers are downloaded, this function recieves the nodes and headers  downloaded
/// and sends a getheaders message to each node to compare and get a header that was not downloaded.
/// it returns error in case of failure.
fn compare_and_ask_for_last_headers(
    config: Arc<Config>,
    log_sender: LogSender,
    nodes: Arc<RwLock<Vec<TcpStream>>>,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
) -> Result<Vec<BlockHeader>, DownloadError> {
    // voy guardando los nodos que saco aca para despues agregarlos al puntero
    let mut nodes_vec: Vec<TcpStream> = vec![];
    let mut new_headers = vec![];
    // recorro todos los nodos
    while !nodes
        .read()
        .map_err(|err| DownloadError::LockError(format!("{:?}", err)))?
        .is_empty()
    {
        let mut node = nodes
            .write()
            .map_err(|err| DownloadError::LockError(format!("{:?}", err)))?
            .pop()
            .ok_or("Error no hay mas nodos para comparar y descargar ultimos headers!\n")
            .map_err(|err| DownloadError::CanNotRead(err.to_string()))?;
        let last_header = headers
            .read()
            .map_err(|err| DownloadError::LockError(format!("{:?}", err)))?
            .last()
            .ok_or("Error no hay headers guardados, no tengo para comparar...\n")
            .map_err(|err| DownloadError::CanNotRead(err.to_string()))?
            .hash();
        GetHeadersMessage::build_getheaders_message(config.clone(), vec![last_header])
            .write_to(&mut node)
            .map_err(|err| DownloadError::WriteNodeError(err.to_string()))?;
        let headers_read = match HeadersMessage::read_from(log_sender.clone(), &mut node, None) {
            Ok(headers) => headers,
            Err(err) => {
                write_in_log(
                    log_sender.error_log_sender.clone(),
                    format!("Error al tratar de leer nuevos headers, descarto nodo. Error: {err}")
                        .as_str(),
                );
                continue;
            }
        };
        // si se recibio un header nuevo lo agrego
        if !headers_read.is_empty() {
            headers
                .write()
                .map_err(|err| DownloadError::LockError(format!("{:?}", err)))?
                .extend_from_slice(&headers_read);
            write_in_log(
                &log_sender.info_log_sender,
                format!(
                    "{} headers encontrados al comparar el ultimo mio con el nodo: {:?}",
                    headers_read.len(),
                    node
                )
                .as_str(),
            );
            new_headers.extend_from_slice(&headers_read);
        }
        nodes_vec.push(node);
    }
    // devuelvo todos los nodos a su puntero
    nodes
        .write()
        .map_err(|err| DownloadError::LockError(format!("{:?}", err)))?
        .extend(nodes_vec);
    Ok(new_headers)
}
*/
