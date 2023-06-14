
use crate::{
    blocks::{block::Block, block_header::BlockHeader},
    listener::listen_for_incoming_messages,
    logwriter::log_writer::{write_in_log, LogSender},
    messages::{
        block_message::BlockMessage, get_data_message::{GetDataMessage, self},
        headers_message::{is_terminated, HeadersMessage}, inventory::Inventory, message_header::{HeaderMessage, get_checksum}, payload,
    },
    transactions::transaction::Transaction,
    wallet::Wallet, compact_size_uint::CompactSizeUint,
};
use std::{
    error::Error,
    fmt,
    net::{TcpStream, SocketAddr},
    sync::{Arc, Mutex, RwLock, mpsc::{Receiver, channel, Sender}},
    thread::{self, JoinHandle}, io::{Write, Read},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NodeMessageHandlerError {
    ThreadJoinError(String),
    LockError(String),
    ReadNodeError(String),
    WriteNodeError(String),
    CanNotRead(String),
    ThreadChannelError(String),
    UnmarshallingError(String),
}

impl fmt::Display for NodeMessageHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeMessageHandlerError::ThreadJoinError(msg) => write!(f, "ThreadJoinError Error: {}", msg),
            NodeMessageHandlerError::LockError(msg) => write!(f, "LockError Error: {}", msg),
            NodeMessageHandlerError::ReadNodeError(msg) => {
                write!(f, "Can not read from socket Error: {}", msg)
            }
            NodeMessageHandlerError::WriteNodeError(msg) => {
                write!(f, "Can not write in socket Error: {}", msg)
            }
            NodeMessageHandlerError::CanNotRead(msg) => {
                write!(f, "No more elements in list Error: {}", msg)
            }
            NodeMessageHandlerError::ThreadChannelError(msg) => {
                write!(f, "Can not send elements to channel Error: {}", msg)
            }
            NodeMessageHandlerError::UnmarshallingError(msg) => {
                write!(f, "Can not unmarshall bytes Error: {}", msg)
            }
        }
    }
}

impl Error for NodeMessageHandlerError {}

type NodeMessageHandlerResult = Result<(), NodeMessageHandlerError>;
type NodeSender = Sender<Vec<u8>>;
type NodeReceiver = Receiver<Vec<u8>>;

#[derive(Debug, Clone)]
pub struct NodeMessageHandler {
    nodes_handle: Arc<Mutex<Vec<JoinHandle<NodeMessageHandlerResult>>>>,
    nodes_sender: Vec<NodeSender>,
    finish: Arc<RwLock<bool>>,
}



impl NodeMessageHandler {
    pub fn new(
        log_sender: LogSender,
        headers: Arc<RwLock<Vec<BlockHeader>>>,
        blocks: Arc<RwLock<Vec<Block>>>,
        connected_nodes: Arc<RwLock<Vec<TcpStream>>>,
    ) -> Result<Self, NodeMessageHandlerError> {
        write_in_log(
            log_sender.info_log_sender.clone(),
            "Empiezo a escuchar por nuevos bloques",
        );
        let finish = Arc::new(RwLock::new(false));
        let mut nodes_handle: Vec<JoinHandle<NodeMessageHandlerResult>> = vec![];
        let cant_nodos = get_amount_of_nodes(connected_nodes.clone())?;
        let nodes_sender = vec![];
        // lista de transacciones recibidas para no recibir las mismas de varios nodos
        let transactions_recieved: Arc<RwLock<Vec<[u8; 32]>>> = Arc::new(RwLock::new(Vec::new()));
        for _ in 0..cant_nodos {
            let (tx, rx) = channel();
            nodes_sender.push(tx.clone());
            let node = get_last_node(connected_nodes.clone())?;
            println!(
                "Nodo -{:?}- Escuchando por nuevos bloques...\n",
                node.peer_addr()
            );
            nodes_handle.push(handle_messages_from_node(
                log_sender.clone(),
                tx,
                rx,
                headers.clone(),
                blocks.clone(),
                transactions_recieved.clone(),
                node,
                Some(finish.clone()),
            ))
        }
        let nodes_handle_mutex = Arc::new(Mutex::new(nodes_handle));
        Ok(NodeMessageHandler {
            nodes_handle: nodes_handle_mutex,
            nodes_sender,
            finish,
        })
    }

    pub fn broadcast_to_nodes(&self, message: Vec<u8>) -> NodeMessageHandlerResult {
        for node_sender in &self.nodes_sender {
            node_sender.send(message.clone()).map_err(|err| NodeMessageHandlerError::ThreadChannelError(err.to_string()))?;
        }
        Ok(())
    }

    pub fn finish(&self) -> NodeMessageHandlerResult {
        *self
            .finish
            .write()
            .map_err(|err| NodeMessageHandlerError::LockError(err.to_string()))? = true;
        let cant_nodos = self
            .nodes_handle
            .lock()
            .map_err(|err| NodeMessageHandlerError::LockError(err.to_string()))?
            .len();
        for _ in 0..cant_nodos {
            self.nodes_handle
                .lock()
                .map_err(|err| NodeMessageHandlerError::LockError(err.to_string()))?
                .pop()
                .ok_or("Error no hay mas nodos para descargar los headers!\n")
                .map_err(|err| NodeMessageHandlerError::CanNotRead(err.to_string()))?
                .join()
                .map_err(|err| NodeMessageHandlerError::ThreadJoinError(format!("{:?}", err)))??;
        }
        for node_sender in self.nodes_sender {
            drop(node_sender);
        }
        Ok(())
    }
}




pub fn handle_messages_from_node(
    log_sender: LogSender,
    tx: NodeSender,
    rx: NodeReceiver,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
    blocks: Arc<RwLock<Vec<Block>>>,
    transactions_recieved: Arc<RwLock<Vec<[u8; 32]>>>,
    mut node: TcpStream,
    finish: Option<Arc<RwLock<bool>>>,
) -> JoinHandle<NodeMessageHandlerResult> {
    let log_sender_clone = log_sender.clone();
    thread::spawn(move || -> NodeMessageHandlerResult {
        while !is_terminated(finish.clone()) {
            // veo si mandaron algo para escribir
            if let Ok(message) = rx.try_recv() {
                write_message_in_node(&mut node, &message)?
            }
            let (header, payload) = read_header_and_payload(&mut node)?;
            let command_name = get_header_command_name_as_str(&header.command_name.as_str());
            match command_name {
                "headers" => {
                    handle_headers_message(log_sender.clone(), tx.clone(), payload, headers.clone())?;
                }
                "getdata" => {
                    //handle_getdata_message()
                }
                "block" => {
                    handle_block_message(log_sender.clone(), payload, headers.clone(), blocks.clone())?;
                }
                "inv" => {
                    handle_inv_message(tx.clone(), payload, transactions_recieved.clone())?;
                }
                "ping" => {
                    handle_ping_message(tx.clone(), payload);
                }
                "tx" => {
                    handle_tx_message(log_sender.clone(), payload);
                }
                _ => {
                    write_in_log(log_sender.messege_log_sender, format!("IGNORADO -- Recibo: {} -- Nodo: {:?}", command_name, node.peer_addr()).as_str());   
                    continue;
                }
            }
            write_in_log(log_sender.messege_log_sender, format!("Recibo correctamente: {:?} -- Nodo: {:?}", command_name, node.peer_addr()).as_str());   
        
        }


        Ok(())
    })
}






fn handle_headers_message(log_sender: LogSender, tx: NodeSender, payload: &[u8], headers: Arc<RwLock<Vec<BlockHeader>>>) -> NodeMessageHandlerResult {
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


fn handle_block_message(log_sender: LogSender, payload: &[u8], headers: Arc<RwLock<Vec<BlockHeader>>>, blocks: Arc<RwLock<Vec<Block>>>) -> NodeMessageHandlerResult {
    let new_block = BlockMessage::unmarshalling(&payload.to_vec()).map_err(|err| NodeMessageHandlerError::UnmarshallingError(err.to_string()))?;
    if new_block.validate().0 {
        let header_is_not_included_yet = header_is_not_included(new_block.block_header, headers.clone())?;
        if header_is_not_included_yet {
            include_new_header(log_sender.clone(), new_block.block_header, headers)?;
            include_new_block(log_sender, new_block, blocks)?;
            // todo: new_block.contains_pending_tx(pending_transactions, confirmed_transactions)?;
        }
    } else {
        write_in_log(
            log_sender.error_log_sender,
            "NUEVO BLOQUE ES INVALIDO, NO LO AGREGO!",
        );
    }
    Ok(())
}



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




























/*
***************************************************************************
****************************** HANDLERS ***********************************
***************************************************************************
*/
/// recieves a NodeSender and the payload of the inv message and creates the inventories to ask for the incoming
/// txs the node sent via inv. Returns error in case of failure or Ok(())
fn handle_inv_message(
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

/// Recibe un NodeSender y un payload y manda por el channel el pong message correspondiente para que se escriba por el nodo
/// y quede respondido el ping. Devuelve Ok(()) en caso de que se pueda enviar bien por el channel o Error de channel en caso contrario
pub fn handle_ping_message(
    tx: NodeSender,
    payload: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
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

/// Recibe un LogSender y el Payload del mensaje tx. Se fija si la tx involucra una cuenta de nuestra wallet. Devuelve Ok(()) 
/// en caso de que se pueda leer bien el payload y recorrer las tx o error en caso contrario
fn handle_tx_message(log_sender: LogSender, payload: &[u8]) -> NodeMessageHandlerResult {
    let tx = Transaction::unmarshalling(&payload.to_vec(), &mut 0).map_err(|err| NodeMessageHandlerError::UnmarshallingError(err.to_string()))?;
    // todo: tx.check_if_tx_involves_user_account(wallet.clone(), pending_transactions.clone())?;
    Ok(())
}





/// Recibe un &str que representa el nombre de un comando de un header con su respectivo nombre
/// y los \0 hasta completar los 12 bytes. Devuelve un &str con el nombre del mensaje y le quita los
/// \0 extras
fn get_header_command_name_as_str(command: &str) -> &str {
    if let Some(first_null_char) = command.find('\0') {
        &command[0..first_null_char]
    } else {
        command
    }
}

/// Recibe algo que implemente el trait Write y un vector de bytes que representa un mensaje. Lo escribe y devuevle
/// Ok(()) en caso de que se escriba exitosamente o un error especifico de escritura en caso contrarios
pub fn write_message_in_node(node: &mut dyn Write, message: &[u8]) -> NodeMessageHandlerResult {
    println!("RECIBI LA TRANSACCION, LA VOY A MANDAR POR EL NODO!\n");
    node.write_all(message).map_err(|err| NodeMessageHandlerError::WriteNodeError(err.to_string()))?;
    node.flush().map_err(|err| NodeMessageHandlerError::WriteNodeError(err.to_string()))?;
    println!("TRANSACCION ENVIADA CORRECTAMENTE!\n");
    Ok(())
}


/// Recibe algo que implemente el trait Read y lee el header y el payload del header y devuelve una tupla de un Struct
/// Header y un vector de bytes que representa al payload. En caso de error al leer, devuelve un Error especifico de lectura
pub fn read_header_and_payload(node: &mut dyn Read) -> Result<(HeaderMessage, &[u8]), NodeMessageHandlerError> {
    let mut buffer_num = [0; 24];
    node.read_exact(&mut buffer_num).map_err(|err| NodeMessageHandlerError::ReadNodeError(err.to_string()))?;
    let mut header = HeaderMessage::from_le_bytes(buffer_num).map_err(|err| NodeMessageHandlerError::ReadNodeError(err.to_string()))?;
    let payload_size = header.payload_size as usize;
    let mut payload_buffer_num: Vec<u8> = vec![0; payload_size];
    node.read_exact(&mut payload_buffer_num).map_err(|err| NodeMessageHandlerError::ReadNodeError(err.to_string()))?;
    Ok((header, &payload_buffer_num))
}


/// Recibe un Arc apuntando a un RwLock de un vector de TcpStreams y devuelve el ultimo nodo TcpStream del vector si es que
/// hay, si no devuelve un error del tipo BroadcastingError
fn get_last_node(nodes: Arc<RwLock<Vec<TcpStream>>>) -> Result<TcpStream, NodeMessageHandlerError> {
    let node = nodes
        .try_write()
        .map_err(|err| NodeMessageHandlerError::LockError(err.to_string()))?
        .pop()
        .ok_or("Error no hay mas nodos para descargar los headers!\n")
        .map_err(|err| NodeMessageHandlerError::CanNotRead(err.to_string()))?;
    Ok(node)
}

/// Recibe un Arc apuntando a un vector de TcpStream y devuelve el largo del vector
fn get_amount_of_nodes(nodes: Arc<RwLock<Vec<TcpStream>>>) -> Result<usize, NodeMessageHandlerError> {
    let amount_of_nodes = nodes
        .read()
        .map_err(|err| NodeMessageHandlerError::LockError(err.to_string()))?
        .len();
    Ok(amount_of_nodes)
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