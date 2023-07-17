use crate::{
    blocks::{block::Block, block_header::BlockHeader},
    config::Config,
    custom_errors::NodeCustomErrors,
    logwriter::log_writer::{write_in_log, LogSender},
    messages::{
        block_message::BlockMessage, get_data_message::GetDataMessage, inventory::Inventory,
    },
};
use std::{
    collections::HashMap,
    net::TcpStream,
    sync::{
        mpsc::{Receiver, Sender},
        Arc, RwLock,
    },
    thread,
};

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
pub fn download_blocks(
    config: &Arc<Config>,
    log_sender: &LogSender,
    nodes: Arc<RwLock<Vec<TcpStream>>>,
    blocks: Arc<RwLock<HashMap<[u8; 32], Block>>>,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
    rx: Receiver<Vec<BlockHeader>>,
    tx: Sender<Vec<BlockHeader>>,
) -> Result<(), NodeCustomErrors> {
    // recieves in the channel the vec of headers sent by the function downloading headers
    for received in rx {
        // acá recibo 2000 block headers
        let mut n_threads = config.n_threads;
        if received.len() < config.blocks_download_per_node {
            n_threads = 1;
        }
        if received.is_empty() {
            return Err(NodeCustomErrors::ThreadChannelError(
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
            let tx_cloned = tx.clone();
            let nodes_pointer_clone = nodes.clone();
            let block_headers_chunk_clone = Arc::clone(&blocks_headers_chunks);
            let blocks_pointer_clone = Arc::clone(&blocks);
            let node = nodes_pointer_clone
                .write()
                .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
                .pop()
                .ok_or("Error no hay mas nodos para descargar los bloques!\n")
                .map_err(|err| NodeCustomErrors::CanNotRead(err.to_string()))?;

            if i >= block_headers_chunk_clone
                .read()
                .map_err(|err| NodeCustomErrors::CanNotRead(err.to_string()))?
                .len()
            {
                // Este caso evita acceder a una posición fuera de rango
                // Significa que no hay más chunks con bloques para descargar
                break;
            }

            let block_headers = block_headers_chunk_clone
                .write()
                .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?[i]
                .clone();
            let log_sender_clone = log_sender.clone();
            let config_cloned = config.clone();
            handle_join.push(thread::spawn(move || {
                download_blocks_single_thread(
                    &config_cloned,
                    &log_sender_clone,
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
                .map_err(|err| NodeCustomErrors::ThreadJoinError(format!("{:?}", err)))??;
        }
        let bloques_descargados = blocks
            .read()
            .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
            .len();
        let cantidad_headers_descargados = headers
            .read()
            .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
            .len();
        let bloques_a_descargar =
            cantidad_headers_descargados - config.height_first_block_to_download + 1;
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
    config: &Arc<Config>,
    log_sender: &LogSender,
    block_headers: Vec<BlockHeader>,
    mut node: TcpStream,
    tx: Sender<Vec<BlockHeader>>,
    blocks_pointer_clone: Arc<RwLock<HashMap<[u8; 32], Block>>>,
    nodes_pointer_clone: Arc<RwLock<Vec<TcpStream>>>,
) -> Result<(), NodeCustomErrors> {
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
            log_sender,
            &node,
            chunk_llamada,
            &block_headers_thread,
            tx.clone(),
        ) {
            Ok(_) => {}
            Err(NodeCustomErrors::WriteNodeError(_)) => return Ok(()),
            Err(error) => return Err(error),
        }
        let received_blocks;
        (node, received_blocks) = match receive_requested_blocks_from_node(
            log_sender,
            node,
            chunk_llamada,
            &block_headers_thread,
            tx.clone(),
        ) {
            Ok(blocks) => blocks,
            Err(NodeCustomErrors::ReadNodeError(_)) => return Ok(()),
            Err(error) => return Err(error),
        };

        for block in received_blocks.into_iter() {
            current_blocks.insert(block.hash(), block);
        }
    }
    blocks_pointer_clone
        .write()
        .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
        .extend(current_blocks);
    write_in_log(
        &log_sender.info_log_sender,
        format!(
            "BLOQUES DESCARGADOS: {:?}",
            blocks_pointer_clone
                .read()
                .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
                .len()
        )
        .as_str(),
    );
    println!(
        "{:?} bloques descargados",
        blocks_pointer_clone
            .read()
            .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
            .len()
    );
    nodes_pointer_clone
        .write()
        .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
        .push(node);
    Ok(())
}

/// Requests the blocks to the node.
/// ## Errors
/// In case of error while sending the message, it returns the block headers back to the channel so
/// they can be downloaded from another node. If this cannot be done, returns an error.
fn request_blocks_from_node(
    log_sender: &LogSender,
    mut node: &TcpStream,
    chunk_llamada: &[BlockHeader],
    block_headers_thread: &[BlockHeader],
    tx: Sender<Vec<BlockHeader>>,
) -> Result<(), NodeCustomErrors> {
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
                .map_err(|err| NodeCustomErrors::ThreadChannelError(err.to_string()))?;
            // falló el envio del mensaje, tengo que intentar con otro nodo
            // si hago return, termino el thread.
            // tengo que enviar todos los bloques que tenía ese thread
            Err(NodeCustomErrors::WriteNodeError(format!("{:?}", err)))
        }
    }
}

/// Receives the blocks previously requested to the node.
/// Returns an array with the blocks.
/// In case of error while receiving the message, it returns the block headers back to the channel so
/// they can be downloaded from another node. If this cannot be done, returns an error.
fn receive_requested_blocks_from_node(
    log_sender: &LogSender,
    mut node: TcpStream,
    chunk_llamada: &[BlockHeader],
    block_headers_thread: &[BlockHeader],
    tx: Sender<Vec<BlockHeader>>,
) -> Result<(TcpStream, Vec<Block>), NodeCustomErrors> {
    // Acá tengo que recibir los 16 bloques (o menos) de la llamada
    let mut current_blocks: Vec<Block> = Vec::new();
    for _ in 0..chunk_llamada.len() {
        let bloque = match BlockMessage::read_from(log_sender, &mut node) {
            Ok(bloque) => bloque,
            Err(err) => {
                write_in_log(&log_sender.error_log_sender,format!("No puedo descargar {:?} de bloques del nodo: {:?}. Se los voy a pedir a otro nodo y descarto este. Error: {err}", chunk_llamada.len(), node.peer_addr()).as_str());
                tx.send(block_headers_thread.to_vec())
                    .map_err(|err| NodeCustomErrors::ThreadChannelError(err.to_string()))?;
                // falló la recepción del mensaje, tengo que intentar con otro nodo
                // termino el nodo con el return
                return Err(NodeCustomErrors::ReadNodeError(format!(
                    "Error al recibir el mensaje `block`: {:?}",
                    err
                )));
            }
        };
        let validation_result = bloque.validate();
        if !validation_result.0 {
            write_in_log(&log_sender.error_log_sender,format!("El bloque no pasó la validación. {:?}. Se los voy a pedir a otro nodo y descarto este.", validation_result.1).as_str());
            tx.send(block_headers_thread.to_vec())
                .map_err(|err| NodeCustomErrors::ThreadChannelError(err.to_string()))?;
            return Err(NodeCustomErrors::ReadNodeError(format!(
                "Error al recibir el mensaje `block`: {:?}",
                validation_result.1
            )));
        }
        //bloque.set_utxos(); // seteo utxos de las transacciones del bloque
        current_blocks.push(bloque);
    }
    Ok((node, current_blocks))
}
