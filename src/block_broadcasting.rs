
use crate::{
    block::Block,
    block_header::BlockHeader,
    log_writer::{write_in_log, LogSender},
    messages::{
        headers_message::{HeadersMessage, is_terminated},
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

type BroadcastingResult = Result<(), BroadcastingError>;

#[derive(Debug)]
pub struct BlockBroadcasting {
    nodes_handle: Arc<Mutex<Vec<JoinHandle<BroadcastingResult>>>>,
    finish: Arc<RwLock<bool>>
}

impl BlockBroadcasting {
    
    pub fn listen_for_incoming_blocks(
        log_sender: LogSender,
        mut nodes: Arc<RwLock<Vec<TcpStream>>>,
        headers: Vec<BlockHeader>,
        blocks: Vec<Block>,
    ) -> Self {
        let finish = Arc::new(RwLock::new(false));
        let mut nodes_handle: Vec<JoinHandle<BroadcastingResult>> = vec![];
        println!("cantidad de nodos: {:?}", nodes.read().unwrap().len());
        let cant_nodos = nodes.read().unwrap().len();
        for _ in 0..cant_nodos {
            let node = nodes.try_write().unwrap().pop().unwrap();
            nodes_handle.push(listen_for_incoming_blocks_from_node(
                log_sender.clone(),
                node,
                headers.clone(),
                blocks.clone(),
                finish.clone(),
            ))
        }
        let nodes_handle_mutex = Arc::new(Mutex::new(nodes_handle));
        BlockBroadcasting {
            nodes_handle: nodes_handle_mutex,
            finish,
        }
    } 
    pub fn finish(&self) -> BroadcastingResult {
        *self.finish.write().unwrap() = true;
        let cant_nodos = self.nodes_handle.lock().unwrap().len();
        for _ in 0..cant_nodos  {
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
    finish: Arc<RwLock<bool>>
) -> JoinHandle<BroadcastingResult> {
    let log_sender_clone = log_sender.clone();
    let t = thread::spawn(move || -> BroadcastingResult {
        while !is_terminated(Some(finish.clone())) {
            println!("Estoy esperando leer algo\n");
            let new_headers = match HeadersMessage::read_from(log_sender_clone.clone(), &mut node, Some(finish.clone())){
              Ok(headers) => headers,
              Err(err) => {
                write_in_log(
                    log_sender.error_log_sender.clone(),
                    format!("Error al recibir headers durante broadcasting. Error: {}", err).as_str(),
                );
                continue;
              }  
            };
            if is_terminated(Some(finish.clone())){
                return Ok(());
            }
            for header in new_headers {
                if !header.validate() {
                    write_in_log(
                        log_sender.error_log_sender.clone(),
                        "Error en validacion de la proof of work de header",
                    );
                } else {
                    let last_header = headers.last().ok_or("No se pudo obtener el último header")
                    .map_err(|err| BroadcastingError::CanNotRead(err.to_string()))?;
                    if *last_header != header {
                        println!("%%%%%%%    HEADERS SON DISTINTOS!!!    %%%%%%%")
                    }
                }
            }
            
            // pedir bloques

        }
        Ok(())
    });
    t
}

