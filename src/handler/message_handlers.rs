use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc, RwLock},
};

use crate::{
    account::Account,
    blocks::{block::Block, block_header::BlockHeader},
    compact_size_uint::CompactSizeUint,
    logwriter::log_writer::{write_in_log, LogSender},
    messages::{
        block_message::{get_block_message, BlockMessage},
        get_data_message::GetDataMessage,
        getheaders_message::GetHeadersMessage,
        headers_message::HeadersMessage,
        inventory::Inventory,
        message_header::{get_checksum, HeaderMessage},
        notfound_message::get_notfound_message,
        payload::get_data_payload::unmarshalling,
    },
    transactions::transaction::Transaction,
    utxo_tuple::UtxoTuple,
};

use crate::custom_errors::NodeCustomErrors;

type NodeMessageHandlerResult = Result<(), NodeCustomErrors>;
type NodeSender = Sender<Vec<u8>>;

const START_STRING: [u8; 4] = [0x0b, 0x11, 0x09, 0x07];
const MSG_TX: u32 = 1;
const MSG_BLOCK: u32 = 2;

/*
***************************************************************************
****************************** HANDLERS ***********************************
***************************************************************************
*/

/// Deserializa el payload del mensaje headers y en caso de ser validos se fijan si no estan incluidos en la cadena de headers. En caso
/// de no estarlo, manda por el channel que escribe en el nodo el mensaje getData con el bloque a pedir
pub fn handle_headers_message(
    log_sender: &LogSender,
    tx: NodeSender,
    payload: &[u8],
    headers: Arc<RwLock<Vec<BlockHeader>>>,
) -> NodeMessageHandlerResult {
    let new_headers = HeadersMessage::unmarshalling(&payload.to_vec())
        .map_err(|err| NodeCustomErrors::UnmarshallingError(err.to_string()))?;
    for header in new_headers {
        if !header.validate() {
            write_in_log(
                &log_sender.error_log_sender,
                "Error en validacion de la proof of work de nuevo header",
            );
        } else {
            // se fija que el header que recibio no este ya incluido en la cadena de headers (con verificar los ultimos 10 alcanza)
            let header_not_included = header_is_not_included(header, headers.clone())?;
            if header_not_included {
                let get_data_message =
                    GetDataMessage::new(vec![Inventory::new_block(header.hash())]);
                let get_data_message_bytes = get_data_message.marshalling();
                tx.send(get_data_message_bytes)
                    .map_err(|err| NodeCustomErrors::ThreadChannelError(err.to_string()))?;
            }
        }
    }
    Ok(())
}

pub fn handle_getheaders_message(
    tx: NodeSender,
    payload: &[u8],
    headers: Arc<RwLock<Vec<BlockHeader>>>,
) -> NodeMessageHandlerResult {
    let getheaders_message = GetHeadersMessage::read_from(payload)
        .map_err(|err| NodeCustomErrors::UnmarshallingError(err.to_string()))?;
    let first_header_asked = getheaders_message.payload.locator_hashes[0];
    let stop_hash_provided = getheaders_message.payload.stop_hash != [0u8; 32];
    let mut headers_to_send: Vec<BlockHeader> = Vec::new();
    let amount_of_headers = headers.read().map_err(|err| NodeCustomErrors::LockError(err.to_string()))?.len();
    for header in headers.read().map_err(|err| NodeCustomErrors::LockError(err.to_string()))?.iter() {
        if header.hash() == first_header_asked {
            if !stop_hash_provided {
                // Si no se provee stop_hash, se envian los 2000 headers siguientes al primero
                let mut index = 0;
                while index < 2000 && index < amount_of_headers {
                    let curr_header = headers.read().map_err(|err| NodeCustomErrors::LockError(err.to_string()))?[index];
                    headers_to_send.push(curr_header);
                    index += 1;
                }
            } else {
                // Si se provee stop_hash, se envian todos los headers hasta el stop_hash
                let mut index = 0;
                while index < amount_of_headers {
                    let curr_header = headers.read().map_err(|err| NodeCustomErrors::LockError(err.to_string()))?[index];
                    headers_to_send.push(curr_header);
                    if curr_header.hash() == getheaders_message.payload.stop_hash
                    {
                        break;
                    }
                    index += 1;
                }
            }
        }
    }
    write_to_node(&tx, HeadersMessage::marshalling(headers_to_send))
}

/// Recibe un Sender de bytes, el payload del mensaje getdata recibido y un vector de cuentas de la wallet y deserializa el mensaje getdata que llega
/// y por cada Inventory que pide si esta como pending_transaction en alguna de las cuentas de la wallet se le envia el mensaje tx con la transaccion pedida
/// por el channel para ser escrita. Devuelve Ok(()) en caso exitoso o error de tipo NodeCustomErrors en caso contrarui
pub fn handle_getdata_message(
    log_sender: &LogSender,
    node_sender: NodeSender,
    payload: &[u8],
    blocks: Arc<RwLock<HashMap<[u8; 32], Block>>>,
    accounts: Arc<RwLock<Arc<RwLock<Vec<Account>>>>>,
) -> Result<(), NodeCustomErrors> {
    // idea: mover a GetDataPayload, que devuelva una lista de inventories
    let mut message_to_send: Vec<u8> = Vec::new();
    let inventories = unmarshalling(payload)
        .map_err(|err| NodeCustomErrors::UnmarshallingError(err.to_string()))?;
    let mut notfound_inventories: Vec<Inventory> = Vec::new();
    for inv in inventories {
        if inv.type_identifier == MSG_TX {
            handle_tx_inventory(log_sender, &inv, &accounts, &node_sender)?;
        }
        if inv.type_identifier == MSG_BLOCK {
            handle_block_inventory(
                log_sender,
                &inv,
                &blocks,
                &mut message_to_send,
                &mut notfound_inventories,
            )?;
        }
    }
    if !notfound_inventories.is_empty() {
        // Hay un bloque o mas que no fueron encontrados en la blockchain
        let notfound_message = get_notfound_message(notfound_inventories);
        message_to_send.extend_from_slice(&notfound_message);
    }
    write_to_node(&node_sender, message_to_send)?;
    Ok(())
}

/// Recibe un inventory, un puntero a la cadena de bloques, un puntero al sender del nodo y un puntero al sender de logs.
/// Se fija si el bloque del inventory esta en la blockchain y si es asi lo agrega al mensaje a enviar. Si no esta en la blockchain
/// lo agrega a la lista de inventories notfound. Devuelve Ok(()) en caso de poder agregarlo correctamente o error del tipo NodeHandlerError en caso de no poder.
fn handle_block_inventory(
    log_sender: &LogSender,
    inventory: &Inventory,
    blocks: &Arc<RwLock<HashMap<[u8; 32], Block>>>,
    message_to_send: &mut Vec<u8>,
    notfound_inventories: &mut Vec<Inventory>,
) -> Result<(), NodeCustomErrors> {
    let block_hash = inventory.hash;
    match blocks
        .read()
        .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
        .get(&block_hash)
    {
        Some(block) => {
            message_to_send.extend_from_slice(&get_block_message(block));
        }
        None => {
            write_in_log(
                &log_sender.error_log_sender,
                &format!(
                    "No se encontro el bloque en la blockchain: {}",
                    crate::account::bytes_to_hex_string(&inventory.hash)
                ),
            );
            notfound_inventories.push(inventory.clone());
        }
    }
    Ok(())
}

/// Se fija si la transaccion del inventory esta en alguna de las cuentas de la wallet y si es asi la envia por el channel para que se escriba en el nodo
fn handle_tx_inventory(
    log_sender: &LogSender,
    inventory: &Inventory,
    accounts: &Arc<RwLock<Arc<RwLock<Vec<Account>>>>>,
    node_sender: &NodeSender,
) -> Result<(), NodeCustomErrors> {
    for account in &*accounts
        .read()
        .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
        .read()
        .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
    {
        for tx in &*account
            .pending_transactions
            .read()
            .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
        {
            if tx.hash() == inventory.hash {
                let tx_message = get_tx_message(tx);
                write_to_node(node_sender, tx_message)?;
                write_in_log(
                    &log_sender.info_log_sender,
                    format!("transaccion {:?} enviada", tx.hex_hash()).as_str(),
                );
            }
        }
    }
    Ok(())
}

/// Deserializa el payload del mensaje blocks y en caso de que el bloque es valido y todavia no este incluido, agrega el header a la cadena de headers
/// y el bloque a la cadena de bloques. Se fija si alguna transaccion del bloque involucra a alguna de las cuentas del programa.
pub fn handle_block_message(
    log_sender: &LogSender,
    payload: &[u8],
    headers: Arc<RwLock<Vec<BlockHeader>>>,
    blocks: Arc<RwLock<HashMap<[u8; 32], Block>>>,
    accounts: Arc<RwLock<Arc<RwLock<Vec<Account>>>>>,
    utxo_set: Arc<RwLock<HashMap<[u8; 32], UtxoTuple>>>,
) -> NodeMessageHandlerResult {
    let new_block = BlockMessage::unmarshalling(&payload.to_vec())
        .map_err(|err| NodeCustomErrors::UnmarshallingError(err.to_string()))?;
    if new_block.validate().0 {
        let header_is_not_included_yet =
            header_is_not_included(new_block.block_header, headers.clone())?;
        if header_is_not_included_yet {
            include_new_header(log_sender, new_block.block_header, headers)?;
            include_new_block(log_sender, new_block.clone(), blocks)?;
            new_block.contains_pending_tx(log_sender, accounts.clone())?;
            new_block
                .give_me_utxos(utxo_set.clone())
                .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?;
            update_accounts_utxo_set(accounts, utxo_set)?;
        }
    } else {
        write_in_log(
            &log_sender.error_log_sender,
            "NUEVO BLOQUE ES INVALIDO, NO LO AGREGO!",
        );
    }
    Ok(())
}

/// Recieves a NodeSender and the payload of the inv message and creates the inventories to ask for the incoming
/// txs the node sent via inv. Returns error in case of failure or Ok(())
pub fn handle_inv_message(
    tx: NodeSender,
    payload: &[u8],
    transactions_received: Arc<RwLock<Vec<[u8; 32]>>>,
) -> NodeMessageHandlerResult {
    let mut offset: usize = 0;
    let count = CompactSizeUint::unmarshalling(payload, &mut offset)
        .map_err(|err| NodeCustomErrors::UnmarshallingError(err.to_string()))?;
    let mut inventories = vec![];
    for _ in 0..count.decoded_value() as usize {
        let mut inventory_bytes = vec![0; 36];
        inventory_bytes.copy_from_slice(&payload[offset..(offset + 36)]);
        let inv = Inventory::from_le_bytes(&inventory_bytes);
        if inv.type_identifier == 1
            && !transactions_received
                .read()
                .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
                .contains(&inv.hash())
        {
            transactions_received
                .write()
                .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
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
/// y quede respondido el ping. Devuelve Ok(()) en caso de que se pueda enviar bien por el channel o Error de channel en caso contrario.
pub fn handle_ping_message(tx: NodeSender, payload: &[u8]) -> NodeMessageHandlerResult {
    let header = HeaderMessage {
        start_string: START_STRING,
        command_name: "pong".to_string(),
        payload_size: payload.len() as u32,
        checksum: get_checksum(payload),
    };
    let header_bytes = HeaderMessage::to_le_bytes(&header);
    let mut message: Vec<u8> = Vec::new();
    message.extend_from_slice(&header_bytes);
    message.extend(payload);
    tx.send(message)
        .map_err(|err| NodeCustomErrors::ThreadChannelError(err.to_string()))?;
    Ok(())
}

/// Recibe un LogSender, el Payload del mensaje tx y un puntero a un puntero con las cuentas de la wallet. Se fija si la tx involucra una cuenta de nuestra wallet. Devuelve Ok(())
/// en caso de que se pueda leer bien el payload y recorrer las tx o error en caso contrario
pub fn handle_tx_message(
    log_sender: &LogSender,
    payload: &[u8],
    accounts: Arc<RwLock<Arc<RwLock<Vec<Account>>>>>,
) -> NodeMessageHandlerResult {
    let tx = Transaction::unmarshalling(&payload.to_vec(), &mut 0)
        .map_err(|err| NodeCustomErrors::UnmarshallingError(err.to_string()))?;
    tx.check_if_tx_involves_user_account(log_sender, accounts)?;
    Ok(())
}

/*
***************************************************************************
***************************************************************************
***************************************************************************
*/

/// Receives the inventories with the tx and the sender to write in the node. Sends the getdata message to ask for the tx
fn ask_for_incoming_tx(tx: NodeSender, inventories: Vec<Inventory>) -> NodeMessageHandlerResult {
    let get_data_message = GetDataMessage::new(inventories);
    let get_data_message_bytes = get_data_message.marshalling();
    tx.send(get_data_message_bytes)
        .map_err(|err| NodeCustomErrors::ThreadChannelError(err.to_string()))?;
    Ok(())
}

/// Recibe un bloque a agregar a la cadena y el puntero Arc apuntando a la cadena de bloques y lo agrega.
/// Devuelve Ok(()) en caso de poder agregarlo correctamente o error del tipo NodeHandlerError en caso de no poder.
fn include_new_block(
    log_sender: &LogSender,
    block: Block,
    blocks: Arc<RwLock<HashMap<[u8; 32], Block>>>,
) -> NodeMessageHandlerResult {
    println!("\nRECIBO NUEVO BLOQUE: {} \n", block.hex_hash());
    write_in_log(
        &log_sender.info_log_sender,
        format!("NUEVO BLOQUE AGREGADO: -- {} --", block.hex_hash()).as_str(),
    );
    blocks
        .write()
        .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
        .insert(block.hash(), block);
    Ok(())
}

/// Recibe un header a agregar a la cadena de headers y el Arc apuntando a la cadena de headers y lo agrega
/// Devuelve Ok(()) en caso de poder agregarlo correctamente o error del tipo NodeHandlerError en caso de no poder
fn include_new_header(
    log_sender: &LogSender,
    header: BlockHeader,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
) -> NodeMessageHandlerResult {
    headers
        .write()
        .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
        .push(header);
    write_in_log(
        &log_sender.info_log_sender,
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
) -> Result<bool, NodeCustomErrors> {
    let headers_guard = headers
        .read()
        .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?;
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

/// Actualiza el utxo_set de cada cuenta
fn update_accounts_utxo_set(
    accounts: Arc<RwLock<Arc<RwLock<Vec<Account>>>>>,
    utxo_set: Arc<RwLock<HashMap<[u8; 32], UtxoTuple>>>,
) -> Result<(), NodeCustomErrors> {
    let accounts_lock = accounts
        .read()
        .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?;
    let mut accounts_inner_lock = accounts_lock
        .write()
        .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?;

    for account_lock in accounts_inner_lock.iter_mut() {
        account_lock
            .set_utxos(utxo_set.clone())
            .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?;
    }
    Ok(())
}

// Devuelve el mensaje tx según la transacción recibida
fn get_tx_message(tx: &Transaction) -> Vec<u8> {
    let mut tx_payload = vec![];
    tx.marshalling(&mut tx_payload);
    let header = HeaderMessage::new("tx".to_string(), Some(&tx_payload));
    let mut tx_message = vec![];
    tx_message.extend_from_slice(&header.to_le_bytes());
    tx_message.extend_from_slice(&tx_payload);

    tx_message
}

/// Manda por el channel el mensaje recibido para que se escriba en el nodo
pub fn write_to_node(tx: &NodeSender, message: Vec<u8>) -> NodeMessageHandlerResult {
    tx.send(message)
        .map_err(|err| NodeCustomErrors::ThreadChannelError(err.to_string()))?;
    Ok(())
}
