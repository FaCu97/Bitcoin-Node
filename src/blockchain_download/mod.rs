use crate::custom_errors::NodeCustomErrors;
use self::blocks_download::download_blocks;
use self::headers_download::{get_initial_headers, download_headers};
use super::blocks::block::Block;
use super::blocks::block_header::BlockHeader;
use super::config::Config;
use super::logwriter::log_writer::{write_in_log, LogSender};
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::mpsc::channel;
use std::sync::{Arc, RwLock};
use std::{thread, vec};
mod headers_download;
mod blocks_download;


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
    if config.ibd_single_node {
        // download_full_blockchain_from_single_node(config, log_sender, nodes)
    }
    let headers = vec![];
    let pointer_to_headers = Arc::new(RwLock::new(headers));
    get_initial_headers(
        config,
        log_sender,
        pointer_to_headers.clone(),
        nodes.clone(),
    )?;        
    let blocks: HashMap<[u8; 32], Block> = HashMap::new();
    let pointer_to_blocks = Arc::new(RwLock::new(blocks));
    // channel to comunicate headers download thread with blocks download thread
    let (tx, rx) = channel();
    let tx_cloned = tx.clone();
    let (pointer_to_headers_clone, pointer_to_nodes_clone) =
        (Arc::clone(&pointer_to_headers), Arc::clone(&nodes));
    let log_sender_clone = log_sender.clone();
    let config_cloned = config.clone();
    let headers_thread = thread::spawn(move || {
        download_headers(
            &config_cloned,
            &log_sender_clone,
            pointer_to_nodes_clone,
            pointer_to_headers_clone,
            tx,
        )
    });
    let pointer_to_headers_clone_for_blocks = Arc::clone(&pointer_to_headers);
    let pointer_to_blocks_clone = Arc::clone(&pointer_to_blocks);
    let log_sender_clone = log_sender.clone();
    let config_cloned = config.clone();
    let blocks_thread = thread::spawn(move || {
        download_blocks(
            &config_cloned,
            &log_sender_clone,
            nodes,
            pointer_to_blocks_clone,
            pointer_to_headers_clone_for_blocks,
            rx,
            tx_cloned,
        )
    });
    headers_thread
        .join()
        .map_err(|err| NodeCustomErrors::ThreadJoinError(format!("{:?}", err)))??;
    blocks_thread
        .join()
        .map_err(|err| NodeCustomErrors::ThreadJoinError(format!("{:?}", err)))??;
    let headers = &*pointer_to_headers
        .read()
        .map_err(|err| NodeCustomErrors::LockError(format!("{:?}", err)))?;
    let blocks = &*pointer_to_blocks
        .read()
        .map_err(|err| NodeCustomErrors::LockError(format!("{:?}", err)))?;
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





/* 
fn download_full_blockchain_from_single_node(config: &Arc<Config>, log_sender: &LogSender, nodes: Arc<RwLock<Vec<TcpStream>>>) -> Result<HeadersBlocksTuple, NodeCustomErrors> {
    let headers = vec![];
    let pointer_to_headers = Arc::new(RwLock::new(headers));
    download_headers(
        config,
        log_sender,
        nodes,
        pointer_to_headers,
        None,
    )
    // download blocks
}
*/











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
