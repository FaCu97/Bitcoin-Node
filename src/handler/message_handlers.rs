use std::sync::{RwLock, Arc, mpsc::Sender};

use crate::{logwriter::log_writer::{LogSender, write_in_log}, messages::{headers_message::HeadersMessage, get_data_message::GetDataMessage, inventory::Inventory, block_message::BlockMessage, message_header::{HeaderMessage, get_checksum}}, blocks::{block_header::BlockHeader, block::Block}, compact_size_uint::CompactSizeUint, transactions::transaction::Transaction, account::Account};

use super::node_message_handler::NodeMessageHandlerError;

type NodeMessageHandlerResult = Result<(), NodeMessageHandlerError>;
type NodeSender = Sender<Vec<u8>>;


/*
***************************************************************************
****************************** HANDLERS ***********************************
***************************************************************************
*/

/// Deserializa el payload del mensaje headers y en caso de ser validos se fijan si no estan incluidos en la cadena de headers. En caso
/// de no estarlo, manda por el channel que escribe en el nodo el mensaje getData con el bloque a pedir
pub fn handle_headers_message(log_sender: LogSender, tx: NodeSender, payload: &[u8], headers: Arc<RwLock<Vec<BlockHeader>>>) -> NodeMessageHandlerResult {
    let new_headers = HeadersMessage::unmarshalling(&payload.to_vec()).map_err(|err| NodeMessageHandlerError::UnmarshallingError(err.to_string()))?;
    for header in new_headers {
        if !header.validate() {
            write_in_log(
                log_sender.error_log_sender.clone(),
                "Error en validacion de la proof of work de nuevo header",
            );
        } else {
            // se fija que el header que recibio no este ya incluido en la cadena de headers (con verificar los ultimos 10 alcanza)
            let header_not_included = header_is_not_included(header, headers.clone())?;
            if header_not_included {
                let get_data_message = GetDataMessage::new(vec![Inventory::new_block(header.hash())]);
                let get_data_message_bytes = get_data_message.marshalling();
                tx.send(get_data_message_bytes).map_err(|err| NodeMessageHandlerError::ThreadChannelError(err.to_string()))?;            
            }
        }
    }
    Ok(())
}

/// Deserializa el payload del mensaje blocks y en caso de que el bloque es valido y todavia no este incluido, agrega el header a la cadena de headers
/// y el bloque a la cadena de bloques. Se fija si alguna transaccion del bloque involucra a alguna de las cuentas del programa.
pub fn handle_block_message(log_sender: LogSender, payload: &[u8], headers: Arc<RwLock<Vec<BlockHeader>>>, blocks: Arc<RwLock<Vec<Block>>>, accounts: Arc<RwLock<Arc<RwLock<Vec<Account>>>>>) -> NodeMessageHandlerResult {
    let new_block = BlockMessage::unmarshalling(&payload.to_vec()).map_err(|err| NodeMessageHandlerError::UnmarshallingError(err.to_string()))?;
    if new_block.validate().0 {
        let header_is_not_included_yet = header_is_not_included(new_block.block_header, headers.clone())?;
        if header_is_not_included_yet {
            include_new_header(log_sender.clone(), new_block.block_header, headers)?;
            include_new_block(log_sender.clone(), new_block.clone(), blocks)?;
            new_block.contains_pending_tx(log_sender, accounts)?;
        }
    } else {
        write_in_log(
            log_sender.error_log_sender,
            "NUEVO BLOQUE ES INVALIDO, NO LO AGREGO!",
        );
    }
    Ok(())
}

/// recieves a NodeSender and the payload of the inv message and creates the inventories to ask for the incoming
/// txs the node sent via inv. Returns error in case of failure or Ok(())
pub fn handle_inv_message(
    tx: NodeSender,
    payload: &[u8],
    transactions_received: Arc<RwLock<Vec<[u8; 32]>>>,
) -> NodeMessageHandlerResult {
    let mut offset: usize = 0;
    let count = CompactSizeUint::unmarshalling(payload, &mut offset).map_err(|err| NodeMessageHandlerError::UnmarshallingError(err.to_string()))?;
    let mut inventories = vec![];
    for _ in 0..count.decoded_value() as usize {
        let mut inventory_bytes = vec![0; 36];
        inventory_bytes.copy_from_slice(&payload[offset..(offset + 36)]);
        let inv = Inventory::from_le_bytes(&inventory_bytes);
        if inv.type_identifier == 1
            && !transactions_received
                .read()
                .map_err(|err| NodeMessageHandlerError::LockError(format!("Error al intentar leer puntero a vector de transacciones recibidas. Error: {:?}", err.to_string()).to_string()))?
                .contains(&inv.hash())
        {
            transactions_received
                .write()
                .map_err(|err| NodeMessageHandlerError::LockError(format!("Error al intentar escribir puntero a vector de transacciones recibidas. Error: {:?}", err.to_string()).to_string()))?
                .push(inv.hash());
            inventories.push(inv);
        }
        offset += 36;
    }
    if !inventories.is_empty() {
        ask_for_incoming_tx(tx, inventories)?;
    }
    Ok(())
}

/// Recibe un NodeSender y un payload y manda por el channel el pong message correspondiente para que se escriba por el nodo
/// y quede respondido el ping. Devuelve Ok(()) en caso de que se pueda enviar bien por el channel o Error de channel en caso contrario
pub fn handle_ping_message(
    tx: NodeSender,
    payload: &[u8],
) -> NodeMessageHandlerResult {
    let header = HeaderMessage {
        start_string: [0x0b, 0x11, 0x09, 0x07],
        command_name: "pong".to_string(),
        payload_size: payload.len() as u32,
        checksum: get_checksum(payload),
    };
    let header_bytes = HeaderMessage::to_le_bytes(&header);
    let mut message: Vec<u8> = Vec::new();
    message.extend_from_slice(&header_bytes);
    message.extend(payload);
    tx.send(message).map_err(|err| NodeMessageHandlerError::ThreadChannelError(err.to_string()))?;
    Ok(())
}

/// Recibe un LogSender, el Payload del mensaje tx y un puntero a un puntero con las cuentas de la wallet. Se fija si la tx involucra una cuenta de nuestra wallet. Devuelve Ok(()) 
/// en caso de que se pueda leer bien el payload y recorrer las tx o error en caso contrario
pub fn handle_tx_message(log_sender: LogSender, payload: &[u8], accounts: Arc<RwLock<Arc<RwLock<Vec<Account>>>>>) -> NodeMessageHandlerResult {
    let tx = Transaction::unmarshalling(&payload.to_vec(), &mut 0).map_err(|err| NodeMessageHandlerError::UnmarshallingError(err.to_string()))?;
    tx.check_if_tx_involves_user_account(log_sender, accounts)?;
    Ok(())
}


/*
***************************************************************************
***************************************************************************
***************************************************************************
*/



/// Receives the inventories with the tx and the sender to write in the node. sends the getdata message to ask for the tx
fn ask_for_incoming_tx(
    tx: NodeSender,
    inventories: Vec<Inventory>,
) -> NodeMessageHandlerResult {
    let get_data_message = GetDataMessage::new(inventories);
    let get_data_message_bytes = get_data_message.marshalling();
    tx.send(get_data_message_bytes).map_err(|err| NodeMessageHandlerError::ThreadChannelError(err.to_string()))?;
    Ok(())
}

/// Recibe un bloque a agregar a la cadena de bloques y el Arc apuntando a la cadena de bloques y lo agrega.
/// Devuelve Ok(()) en caso de poder agregarlo correctamente o error del tipo NodeHandlerError en caso de no poder
fn include_new_block(
    log_sender: LogSender,
    block: Block,
    blocks: Arc<RwLock<Vec<Block>>>,
) -> NodeMessageHandlerResult {
    blocks
    .write()
    .map_err(|err| NodeMessageHandlerError::LockError(err.to_string()))?
    .push(block.clone());
    println!("%%%%%%%% RECIBO NUEVO BLOQUE %%%%%%%\n");
    write_in_log(log_sender.info_log_sender, "NUEVO BLOQUE AGREGADO!");
    Ok(())
}

/// Recibe un header a agregar a la cadena de headers y el Arc apuntando a la cadena de headers y lo agrega
/// Devuelve Ok(()) en caso de poder agregarlo correctamente o error del tipo NodeHandlerError en caso de no poder
fn include_new_header(
    log_sender: LogSender,
    header: BlockHeader,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
) -> NodeMessageHandlerResult {
    headers
        .write()
        .map_err(|err| NodeMessageHandlerError::LockError(err.to_string()))?
        .push(header);
    write_in_log(
        log_sender.info_log_sender,
        "Recibo un nuevo header, lo agrego a la cadena de headers!",
    );
    Ok(())
}


/// Recibe un header y la lista de headers y se fija en los ulitmos 10 headers de la lista, si es que existen, que el header
/// no este incluido ya. En caso de estar incluido devuelve false y en caso de nos estar incluido devuelve true. Devuelve error en caso de
/// que no se pueda leer la lista de headers
fn header_is_not_included(
    header: BlockHeader,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
) -> Result<bool, NodeMessageHandlerError> {
    let headers_guard = headers
        .read()
        .map_err(|err| NodeMessageHandlerError::LockError(err.to_string()))?;
    let start_index = headers_guard.len().saturating_sub(10);
    let last_10_headers = &headers_guard[start_index..];
    // Verificar si el header está en los ultimos 10 headers
    for included_header in last_10_headers.iter() {
        if *included_header == header {
            return Ok(false);
        }
    }
    Ok(true)
}