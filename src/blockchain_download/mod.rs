use self::blocks_download::{download_blocks, download_blocks_single_node};
use self::headers_download::{download_missing_headers, get_initial_headers};
use self::utils::{get_amount_of_headers_and_blocks, get_node, join_threads, return_node_to_vec};
use super::blocks::block::Block;
use super::blocks::block_header::BlockHeader;
use super::config::Config;
use super::logwriter::log_writer::{write_in_log, LogSender};
use crate::custom_errors::NodeCustomErrors;
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::mpsc::channel;
use std::sync::{Arc, RwLock};
use std::{thread, vec};
mod blocks_download;
mod headers_download;
mod utils;

type HeadersBlocksTuple = (
    Arc<RwLock<Vec<BlockHeader>>>,
    Arc<RwLock<HashMap<[u8; 32], Block>>>,
);

/// Recieves a list of TcpStreams that are the connection with nodes already established and downloads
/// all the headers from the blockchain and the blocks from a config date. Returns the headers and blocks in
/// two separete lists in case of exit or an error in case of faliure
pub fn initial_block_download(
    config: &Arc<Config>,
    log_sender: &LogSender,
    nodes: Arc<RwLock<Vec<TcpStream>>>,
) -> Result<HeadersBlocksTuple, NodeCustomErrors> {
    write_in_log(
        &log_sender.info_log_sender,
        "EMPIEZA DESCARGA INICIAL DE BLOQUES",
    );
    let headers = vec![];
    let pointer_to_headers = Arc::new(RwLock::new(headers));
    let blocks: HashMap<[u8; 32], Block> = HashMap::new();
    let pointer_to_blocks = Arc::new(RwLock::new(blocks));
    get_initial_headers(
        config,
        log_sender,
        pointer_to_headers.clone(),
        nodes.clone(),
    )?;
    let amount_of_nodes = nodes
        .read()
        .map_err(|err| NodeCustomErrors::LockError(format!("{:?}", err)))?
        .len();
    if config.ibd_single_node || amount_of_nodes < 2 {
        download_full_blockchain_from_single_node(
            config,
            log_sender,
            nodes,
            pointer_to_headers.clone(),
            pointer_to_blocks.clone(),
        )?;
    } else {
        download_full_blockchain_from_multiple_nodes(
            config,
            log_sender,
            nodes,
            pointer_to_headers.clone(),
            pointer_to_blocks.clone(),
        )?;
    }
    let (amount_of_headers, amount_of_blocks) =
        get_amount_of_headers_and_blocks(pointer_to_headers.clone(), pointer_to_blocks.clone())?;
    write_in_log(
        &log_sender.info_log_sender,
        format!("TOTAL DE HEADERS DESCARGADOS: {}", amount_of_headers).as_str(),
    );
    write_in_log(
        &log_sender.info_log_sender,
        format!("TOTAL DE BLOQUES DESCARGADOS: {}\n", amount_of_blocks).as_str(),
    );
    Ok((pointer_to_headers, pointer_to_blocks))
}

/// Se encarga de descargar todos los headers y bloques de la blockchain en multiples thread, en un thread descarga los headers
/// y en el otro a medida que se van descargando los headers va pidiendo los bloques correspondientes.
/// Devuelve error en caso de falla.
fn download_full_blockchain_from_multiple_nodes(
    config: &Arc<Config>,
    log_sender: &LogSender,
    nodes: Arc<RwLock<Vec<TcpStream>>>,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
    blocks: Arc<RwLock<HashMap<[u8; 32], Block>>>,
) -> Result<(), NodeCustomErrors> {
    // channel to comunicate headers download thread with blocks download thread
    let (tx, rx) = channel();
    let mut threads_handle = vec![];
    let config_cloned = config.clone();
    let log_sender_cloned = log_sender.clone();
    let nodes_cloned = nodes.clone();
    let headers_cloned = headers.clone();
    let tx_cloned = tx.clone();
    threads_handle.push(thread::spawn(move || {
        download_missing_headers(
            &config_cloned,
            &log_sender_cloned,
            nodes_cloned,
            headers_cloned,
            tx_cloned,
        )
    }));
    let config = config.clone();
    let log_sender = log_sender.clone();
    threads_handle.push(thread::spawn(move || {
        download_blocks(&config, &log_sender, nodes, blocks, headers, rx, tx)
    }));
    join_threads(threads_handle)?;
    Ok(())
}

/// Se encarga de descargar todos los headers y bloques de la blockchain en un solo thread, primero descarga todos los headers
/// y luego descarga todos los bloques. Devuelve error en caso de falla.
fn download_full_blockchain_from_single_node(
    config: &Arc<Config>,
    log_sender: &LogSender,
    nodes: Arc<RwLock<Vec<TcpStream>>>,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
    blocks: Arc<RwLock<HashMap<[u8; 32], Block>>>,
) -> Result<(), NodeCustomErrors> {
    let (tx, rx) = channel();
    download_missing_headers(config, log_sender, nodes.clone(), headers.clone(), tx)?;
    let mut node = get_node(nodes.clone())?;
    for blocks_to_download in rx {
        download_blocks_single_node(
            config,
            log_sender,
            blocks_to_download,
            &mut node,
            blocks.clone(),
        )?;
    }
    return_node_to_vec(nodes, node)?;
    Ok(())
}

/*
/// Once the headers are downloaded, this function recieves the nodes and headers  downloaded
/// and sends a getheaders message to each node to compare and get a header that was not downloaded.
/// it returns error in case of failure.
fn compare_and_ask_for_last_headers(
    config: &Arc<Config>,
    log_sender: &LogSender,
    nodes: Arc<RwLock<Vec<TcpStream>>>,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
) -> Result<Vec<BlockHeader>, NodeCustomErrors> {
    // voy guardando los nodos que saco aca para despues agregarlos al puntero
    let mut nodes_vec: Vec<TcpStream> = vec![];
    let mut new_headers = vec![];
    // recorro todos los nodos
    while !nodes
        .read()
        .map_err(|err| NodeCustomErrors::LockError(format!("{:?}", err)))?
        .is_empty()
    {
        let mut node = nodes
            .write()
            .map_err(|err| NodeCustomErrors::LockError(format!("{:?}", err)))?
            .pop()
            .ok_or("Error no hay mas nodos para comparar y descargar ultimos headers!\n")
            .map_err(|err| NodeCustomErrors::CanNotRead(err.to_string()))?;
        let last_header = headers
            .read()
            .map_err(|err| NodeCustomErrors::LockError(format!("{:?}", err)))?
            .last()
            .ok_or("Error no hay headers guardados, no tengo para comparar...\n")
            .map_err(|err| NodeCustomErrors::CanNotRead(err.to_string()))?
            .hash();
        GetHeadersMessage::build_getheaders_message(config, vec![last_header])
            .write_to(&mut node)
            .map_err(|err| NodeCustomErrors::WriteNodeError(err.to_string()))?;
        let headers_read = match HeadersMessage::read_from(log_sender, &mut node, None) {
            Ok(headers) => headers,
            Err(err) => {
                write_in_log(
                    &log_sender.error_log_sender,
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
                .map_err(|err| NodeCustomErrors::LockError(format!("{:?}", err)))?
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
        .map_err(|err| NodeCustomErrors::LockError(format!("{:?}", err)))?
        .extend(nodes_vec);
    Ok(new_headers)
}
*/
