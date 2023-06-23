use crate::{
    account::Account,
    blocks::{block::Block, block_header::BlockHeader},
    logwriter::log_writer::{write_in_log, LogSender},
    messages::{headers_message::is_terminated, message_header::HeaderMessage},
    utxo_tuple::UtxoTuple,
};
use std::{
    collections::HashMap,
    error::Error,
    fmt,
    io::{self, Read, Write},
    net::TcpStream,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex, RwLock,
    },
    thread::{self, JoinHandle},
};

use super::message_handlers::{
    handle_block_message, handle_getdata_message, handle_headers_message, handle_inv_message,
    handle_ping_message, handle_tx_message,
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
            NodeMessageHandlerError::ThreadJoinError(msg) => {
                write!(f, "ThreadJoinError Error: {}", msg)
            }
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
type NodeBlocksData = (Arc<RwLock<Vec<BlockHeader>>>, Arc<RwLock<Vec<Block>>>);

#[derive(Debug, Clone)]
/// Struct para controlar todos los nodos conectados al nuestro. Escucha permanentemente
/// a estos y decide que hacer con los mensajes que llegan y con los que tiene que escribir
pub struct NodeMessageHandler {
    nodes_handle: Arc<Mutex<Vec<JoinHandle<NodeMessageHandlerResult>>>>,
    nodes_sender: Vec<NodeSender>,
    finish: Arc<RwLock<bool>>,
}

impl NodeMessageHandler {
    /// Recibe la informacion que tiene el nodo (headers, bloques y nodos conectados)
    /// y se encarga de crear un thread por cada nodo y lo deja esuchando mensajes
    /// y handleandolos de forma oportuna. Si ocurre algun error devuelve un Error del enum
    /// NodeMessageHandlerError y en caso contrario devuelve el nuevo struct
    /// NodeMessageHandler con sus respectivos campos
    pub fn new(
        log_sender: LogSender,
        headers: Arc<RwLock<Vec<BlockHeader>>>,
        blocks: Arc<RwLock<Vec<Block>>>,
        connected_nodes: Arc<RwLock<Vec<TcpStream>>>,
        accounts: Arc<RwLock<Arc<RwLock<Vec<Account>>>>>,
        utxo_set: Arc<RwLock<HashMap<[u8; 32], UtxoTuple>>>,
    ) -> Result<Self, NodeMessageHandlerError> {
        write_in_log(
            log_sender.info_log_sender.clone(),
            "Empiezo a escuchar por nuevos bloques y transaccciones",
        );
        let finish = Arc::new(RwLock::new(false));
        let mut nodes_handle: Vec<JoinHandle<NodeMessageHandlerResult>> = vec![];
        let cant_nodos = get_amount_of_nodes(connected_nodes.clone())?;
        let mut nodes_sender = vec![];
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
                (tx, rx),
                (headers.clone(), blocks.clone()),
                transactions_recieved.clone(),
                accounts.clone(),
                utxo_set.clone(),
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

    /// Recibe un vector de bytes que representa un mensaje serializado y se lo manda a cada canal que esta esperando para escribir en un nodo
    /// De esta manera se broadcastea a todos los nodos conectados el mensaje.
    /// Devuelve Ok(()) en caso exitoso o un error ThreadChannelError en caso contrario
    pub fn broadcast_to_nodes(&self, message: Vec<u8>) -> NodeMessageHandlerResult {
        for node_sender in &self.nodes_sender {
            // si alguno de los channels esta cerrado significa que por alguna razon el nodo fallo entonces lo ignoro y pruebo broadcastear
            // en los siguientes nodos restantes
            if let Err(_) = node_sender.send(message.clone()) {
                continue;
            }
        }
        Ok(())
    }

    /// Se encarga de actualizar el valor del puntero finish que corta los ciclos de los nodos que estan siendo esuchados.
    /// le hace el join a cada uno de los threads por cada nodo que estaba siendo escuchado.
    /// A cada extremo del channel para escribir en los nodos les hace drop() para que se cierre el channel.
    /// Devuelve Ok(()) en caso de salir todo bien o Error especifico en caso contrario
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
        for node_sender in self.nodes_sender.clone() {
            drop(node_sender);
        }
        Ok(())
    }
}

/// Funcion encargada de crear un thread para un nodo especifico y se encarga de realizar el loop que escucha
/// por nuevos mensajes del nodo. En caso de ser necesario tambien escribe al nodo mensajes que le llegan por el channel.
/// El puntero finish define cuando el programa termina y por lo tanto el ciclo de esta funcion. Devuelve el JoinHanfle del thread
/// con lo que devuelve el loop. Ok(()) en caso de salir todo bien o NodeHandlerError en caso de algun error
pub fn handle_messages_from_node(
    log_sender: LogSender,
    (tx, rx): (NodeSender, NodeReceiver),
    (headers, blocks): NodeBlocksData,
    transactions_recieved: Arc<RwLock<Vec<[u8; 32]>>>,
    accounts: Arc<RwLock<Arc<RwLock<Vec<Account>>>>>,
    utxo_set: Arc<RwLock<HashMap<[u8; 32], UtxoTuple>>>,
    mut node: TcpStream,
    finish: Option<Arc<RwLock<bool>>>,
) -> JoinHandle<NodeMessageHandlerResult> {
    thread::spawn(move || -> NodeMessageHandlerResult {
        while !is_terminated(finish.clone()) {
            // veo si mandaron algo para escribir
            if let Ok(message) = rx.try_recv() {
                write_message_in_node(&mut node, &message)?
            }
            //leo header y payload
            let header = read_header(&mut node, finish.clone())?;
            if is_terminated(finish.clone()) {
                break;
            }
            let payload = read_payload(&mut node, header.payload_size as usize, finish.clone())?;
            let command_name = get_header_command_name_as_str(header.command_name.as_str());
            match command_name {
                "headers" => {
                    handle_headers_message(
                        log_sender.clone(),
                        tx.clone(),
                        &payload,
                        headers.clone(),
                    )?;
                }
                "getdata" => {
                    handle_getdata_message(
                        log_sender.clone(),
                        tx.clone(),
                        &payload,
                        accounts.clone(),
                    )?;
                }
                "block" => {
                    handle_block_message(
                        log_sender.clone(),
                        &payload,
                        headers.clone(),
                        blocks.clone(),
                        accounts.clone(),
                        utxo_set.clone(),
                    )?;
                }
                "inv" => {
                    handle_inv_message(tx.clone(), &payload, transactions_recieved.clone())?;
                }
                "ping" => {
                    handle_ping_message(tx.clone(), &payload)?;
                }
                "tx" => {
                    handle_tx_message(log_sender.clone(), &payload, accounts.clone())?;
                }
                _ => {
                    write_in_log(
                        log_sender.messege_log_sender.clone(),
                        format!(
                            "IGNORADO -- Recibo: {} -- Nodo: {:?}",
                            header.command_name,
                            node.peer_addr()
                        )
                        .as_str(),
                    );
                    continue;
                }
            }
            if command_name != "inv" {
                // no me interesa tener todos los inv en el log_message
                write_in_log(
                    log_sender.messege_log_sender.clone(),
                    format!(
                        "Recibo correctamente: {} -- Nodo: {:?}",
                        command_name,
                        node.peer_addr()
                    )
                    .as_str(),
                );
            }
        }
        Ok(())
    })
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
    node.write_all(message)
        .map_err(|err| NodeMessageHandlerError::WriteNodeError(err.to_string()))?;
    node.flush()
        .map_err(|err| NodeMessageHandlerError::WriteNodeError(err.to_string()))?;
    Ok(())
}

fn read_header(
    node: &mut dyn Read,
    finish: Option<Arc<RwLock<bool>>>,
) -> Result<HeaderMessage, NodeMessageHandlerError> {
    let mut buffer_num = [0; 24];
    while !is_terminated(finish.clone()) {
        match node.read_exact(&mut buffer_num) {
            Ok(_) => break, // Lectura exitosa, salimos del bucle
            Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => continue, // No hay suficientes datos disponibles, continuar esperando
            Err(err) => return Err(NodeMessageHandlerError::ReadNodeError(err.to_string())), // Error inesperado, devolverlo
        }
    }
    if is_terminated(finish) {
        // devuelvo un header cualquiera para que no falle en la funcion en la que se llama a read_header
        // y de esta manera cortar bien el ciclo while
        return Ok(HeaderMessage::new("none".to_string(), None));
    }
    HeaderMessage::from_le_bytes(buffer_num)
        .map_err(|err| NodeMessageHandlerError::UnmarshallingError(err.to_string()))
}

fn read_payload(
    node: &mut dyn Read,
    size: usize,
    finish: Option<Arc<RwLock<bool>>>,
) -> Result<Vec<u8>, NodeMessageHandlerError> {
    let mut payload_buffer_num: Vec<u8> = vec![0; size];
    while !is_terminated(finish.clone()) {
        match node.read_exact(&mut payload_buffer_num) {
            Ok(_) => break, // Lectura exitosa, salimos del bucle
            Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => continue, // No hay suficientes datos disponibles, continuar esperando
            Err(err) => return Err(NodeMessageHandlerError::ReadNodeError(err.to_string())), // Error inesperado, devolverlo
        }
    }
    Ok(payload_buffer_num)
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
fn get_amount_of_nodes(
    nodes: Arc<RwLock<Vec<TcpStream>>>,
) -> Result<usize, NodeMessageHandlerError> {
    let amount_of_nodes = nodes
        .read()
        .map_err(|err| NodeMessageHandlerError::LockError(err.to_string()))?
        .len();
    Ok(amount_of_nodes)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn get_header_command_name_as_str_returns_correct_headers_command_name() {
        let header_command_name = "headers\0\0\0\0\0";
        assert_eq!(
            get_header_command_name_as_str(header_command_name),
            "headers"
        );
    }
    #[test]
    fn get_header_command_name_as_str_returns_correct_tx_command_name() {
        let header_command_name = "tx\0\0\0\0\0\0\0\0\0\0";
        assert_eq!(get_header_command_name_as_str(header_command_name), "tx");
    }
}
