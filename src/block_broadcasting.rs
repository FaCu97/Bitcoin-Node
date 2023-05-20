
use crate::{
    block::Block,
    block_header::BlockHeader,
    log_writer::{write_in_log, LogSender},
    messages::{
        headers_message::HeadersMessage,
        message_header::{command_name_to_bytes, HeaderMessage},
    },
    node,
};
use std::{
    fmt,
    io::Read,
    net::TcpStream,
    sync::{Arc, RwLock, Mutex},
    thread::{self, JoinHandle},
    error::Error,
};

const HEADERS: &str = "headers\0\0\0\0\0";

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BroadcastingError {
    ThreadJoinError(String),
    LockError(String),
    ReadNodeError(String),
    WriteNodeError(String),
    CanNotRead(String),
    ThreadChannelError(String),
}

impl fmt::Display for BroadcastingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BroadcastingError::ThreadJoinError(msg) => write!(f, "ThreadJoinError Error: {}", msg),
            BroadcastingError::LockError(msg) => write!(f, "LockError Error: {}", msg),
            BroadcastingError::ReadNodeError(msg) =>  write!(f, "Can not read from socket Error: {}", msg),
            BroadcastingError::WriteNodeError(msg) => write!(f, "Can not write in socket Error: {}", msg),  
            BroadcastingError::CanNotRead(msg) => write!(f, "No more elements in list Error: {}", msg),
            BroadcastingError::ThreadChannelError(msg) => write!(f, "Can not send elements to channel Error: {}", msg),
            
        }
    }
}

impl Error for BroadcastingError {}
#[derive(Debug)]

pub struct BlockBroadcasting {
    nodes_handle: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl BlockBroadcasting {
    
    pub fn listen_for_incoming_blocks(
        log_sender: LogSender,
        mut nodes: Arc<RwLock<Vec<TcpStream>>>,
        headers: Vec<BlockHeader>,
        blocks: Vec<Block>,
    ) -> Self {
        let mut nodes_handle: Vec<JoinHandle<()>> = vec![];
        println!("cantidad de nodos: {:?}", nodes.read().unwrap().len());
        for _ in 0..nodes.read().unwrap().len() {
            let node = nodes.write().unwrap().pop().unwrap();
            nodes_handle.push(listen_for_incoming_blocks_from_node(
                log_sender.clone(),
                node,
                headers.clone(),
                blocks.clone(),
            ))
        }
        let nodes_handle_mutex = Arc::new(Mutex::new(nodes_handle));
        BlockBroadcasting {
            nodes_handle: nodes_handle_mutex,
        }
    } 
    pub fn finish(&self) -> Result<(), BroadcastingError>{
        for i in 0..self.nodes_handle.lock().unwrap().len()  {
            self.nodes_handle.lock().unwrap().pop().unwrap().join().unwrap();
         //   handle.join().unwrap();
        }
        Ok(())
    }
}

    



pub fn listen_for_incoming_blocks_from_node(
    log_sender: LogSender,
    mut node: TcpStream,
    headers: Vec<BlockHeader>,
    blocks: Vec<Block>,
) -> JoinHandle<()> {
    let log_sender_clone = log_sender.clone();
    let t = thread::spawn(move || {
        loop {
            println!("Estoy esperando leer algo\n");
            let header = HeadersMessage::read_from(log_sender_clone.clone(), &mut node).unwrap();
            // pedir bloques
        }
    });
    t
}
