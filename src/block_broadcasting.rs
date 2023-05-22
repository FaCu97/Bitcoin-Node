use crate::{
    blocks::{block::Block, block_header::BlockHeader},
    logwriter::log_writer::{write_in_log, LogSender},
    messages::headers_message::{is_terminated, HeadersMessage},
};
use std::{
    error::Error,
    fmt,
    net::TcpStream,
    sync::{Arc, Mutex, RwLock},
    thread::{self, JoinHandle},
};

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
            BroadcastingError::ReadNodeError(msg) => {
                write!(f, "Can not read from socket Error: {}", msg)
            }
            BroadcastingError::WriteNodeError(msg) => {
                write!(f, "Can not write in socket Error: {}", msg)
            }
            BroadcastingError::CanNotRead(msg) => {
                write!(f, "No more elements in list Error: {}", msg)
            }
            BroadcastingError::ThreadChannelError(msg) => {
                write!(f, "Can not send elements to channel Error: {}", msg)
            }
        }
    }
}

impl Error for BroadcastingError {}

type BroadcastingResult = Result<(), BroadcastingError>;

#[derive(Debug)]
pub struct BlockBroadcasting {
    nodes_handle: Arc<Mutex<Vec<JoinHandle<BroadcastingResult>>>>,
    finish: Arc<RwLock<bool>>,
}

impl BlockBroadcasting {
    pub fn listen_for_incoming_blocks(
        log_sender: LogSender,
        nodes: Arc<RwLock<Vec<TcpStream>>>,
        headers: Arc<RwLock<Vec<BlockHeader>>>,
        blocks: Arc<RwLock<Vec<Block>>>,
    ) -> Result<Self, BroadcastingError> {
        write_in_log(log_sender.info_log_sender.clone(), "Empiezo a escuchar por nuevos bloques");
        let finish = Arc::new(RwLock::new(false));
        let mut nodes_handle: Vec<JoinHandle<BroadcastingResult>> = vec![];
        let cant_nodos = nodes
            .read()
            .map_err(|err| BroadcastingError::LockError(err.to_string()))?
            .len();
        for _ in 0..cant_nodos {
            let node = nodes
                .try_write()
                .map_err(|err| BroadcastingError::LockError(err.to_string()))?
                .pop()
                .ok_or("Error no hay mas nodos para descargar los headers!\n")
                .map_err(|err| BroadcastingError::CanNotRead(err.to_string()))?;
            println!("Nodo -{:?}- Escuchando por nuevos bloques...\n", node.peer_addr());
            nodes_handle.push(listen_for_incoming_blocks_from_node(
                log_sender.clone(),
                node,
                headers.clone(),
                blocks.clone(),
                finish.clone(),
            ))
        }
        let nodes_handle_mutex = Arc::new(Mutex::new(nodes_handle));
        Ok(BlockBroadcasting {
            nodes_handle: nodes_handle_mutex,
            finish,
        })
    }
    pub fn finish(&self) -> BroadcastingResult {
        *self
            .finish
            .write()
            .map_err(|err| BroadcastingError::LockError(err.to_string()))? = true;
        let cant_nodos = self
            .nodes_handle
            .lock()
            .map_err(|err| BroadcastingError::LockError(err.to_string()))?
            .len();
        for _ in 0..cant_nodos {
            self.nodes_handle
                .lock()
                .map_err(|err| BroadcastingError::LockError(err.to_string()))?
                .pop()
                .ok_or("Error no hay mas nodos para descargar los headers!\n")
                .map_err(|err| BroadcastingError::CanNotRead(err.to_string()))?
                .join()
                .map_err(|err| BroadcastingError::ThreadJoinError(format!("{:?}", err)))??;
        }
        Ok(())
    }
}

pub fn listen_for_incoming_blocks_from_node(
    log_sender: LogSender,
    mut node: TcpStream,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
    _blocks: Arc<RwLock<Vec<Block>>>,
    finish: Arc<RwLock<bool>>,
) -> JoinHandle<BroadcastingResult> {
    let log_sender_clone = log_sender.clone();
    thread::spawn(move || -> BroadcastingResult {
        while !is_terminated(Some(finish.clone())) {
            let new_headers = match HeadersMessage::read_from(
                log_sender_clone.clone(),
                &mut node,
                Some(finish.clone()),
            ) {
                Ok(headers) => headers,
                Err(_) => {
                    continue;
                }
            };
            if is_terminated(Some(finish.clone())) {
                return Ok(());
            }
            for header in new_headers {
                if !header.validate() {
                    write_in_log(
                        log_sender.error_log_sender.clone(),
                        "Error en validacion de la proof of work de header",
                    );
                } else {
                    let last_header = *headers
                        .read()
                        .map_err(|err| BroadcastingError::LockError(err.to_string()))?
                        .last()
                        .ok_or("No se pudo obtener el último header")
                        .map_err(|err| BroadcastingError::CanNotRead(err.to_string()))?;
                    if last_header != header {
                       println!("%%%%%%%    Recibo nuevo header!!!    %%%%%%%");
                       headers.write().map_err(|err| BroadcastingError::LockError(err.to_string()))?.push(header);
                       write_in_log(
                        log_sender.info_log_sender.clone(),
                        "Recibo un nuevo header, lo agrego a la cadena de headers!",
                        );
                    }
                }
            }

            // pedir bloques
        }
        Ok(())
    })
}
