use crate::{
    blocks::{block::Block, block_header::BlockHeader},
    logwriter::log_writer::{write_in_log, LogSender},
    messages::{
        block_message::BlockMessage,
        get_data_message::GetDataMessage,
        headers_message::{is_terminated, HeadersMessage},
        inventory::Inventory,
    },
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
        write_in_log(
            log_sender.info_log_sender.clone(),
            "Empiezo a escuchar por nuevos bloques",
        );
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
            println!(
                "Nodo -{:?}- Escuchando por nuevos bloques...\n",
                node.peer_addr()
            );
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
    blocks: Arc<RwLock<Vec<Block>>>,
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
<<<<<<< HEAD
            let cloned_node = node.try_clone().map_err(|err| BroadcastingError::ReadNodeError(err.to_string()))?;
            ask_for_new_blocks(log_sender.clone(), new_headers, cloned_node, headers.clone(), blocks.clone())?;
=======
            for header in new_headers {
                if !header.validate() {
                    write_in_log(
                        log_sender.error_log_sender.clone(),
                        "Error en validacion de la proof of work de nuevo header",
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
                        headers
                            .write()
                            .map_err(|err| BroadcastingError::LockError(err.to_string()))?
                            .push(header);
                        write_in_log(
                            log_sender.info_log_sender.clone(),
                            "Recibo un nuevo header, lo agrego a la cadena de headers!",
                        );
                        if let Err(err) =
                            GetDataMessage::new(vec![Inventory::new_block(header.hash())])
                                .write_to(&mut node)
                        {
                            write_in_log(
                                log_sender.error_log_sender.clone(),
                                format!(
                                    "Error al pedir bloque -{:?}- a nodo -{:?}-. Error: {err}",
                                    header.hash(),
                                    node.peer_addr()
                                )
                                .as_str(),
                            );
                            continue;
                        }
                        let mut new_block = match BlockMessage::read_from(
                            log_sender.clone(),
                            &mut node,
                        ) {
                            Err(err) => {
                                write_in_log(log_sender.error_log_sender.clone(), format!("Error al recibir bloque -{:?}- del nodo -{:?}-. Error: {err}", header.hash(), node.peer_addr()).as_str());
                                continue;
                            }
                            Ok(block) => block,
                        };
                        if new_block.validate().0 {
                            new_block.set_utxos();
                            blocks.write().unwrap().push(new_block);
                            write_in_log(
                                log_sender.info_log_sender.clone(),
                                "NUEVO BLOQUE AGREGADO!",
                            );
                        } else {
                            write_in_log(
                                log_sender.error_log_sender.clone(),
                                "NUEVO BLOQUE ES INVALIDO, NO LO AGREGO!",
                            );
                        }
                    }
                }
            }
>>>>>>> main
        }
        Ok(())
    })
}


pub fn ask_for_new_blocks(log_sender: LogSender, new_headers: Vec<BlockHeader>, node: TcpStream, headers: Arc<RwLock<Vec<BlockHeader>>>, blocks: Arc<RwLock<Vec<Block>>>) -> BroadcastingResult {
    for header in new_headers {
        if !header.validate() {
            write_in_log(
                log_sender.error_log_sender.clone(),
                "Error en validacion de la proof of work de nuevo header",
            );
        } else {
            let last_header = get_last_header(headers.clone())?;
            if last_header != header {
                recieve_new_header(log_sender.clone(), header, headers.clone())?;
                let cloned_node = node.try_clone().map_err(|err| BroadcastingError::ReadNodeError(err.to_string()))?;
                if let Err(_) = ask_for_new_block(log_sender.clone(), cloned_node, header) {
                    continue;
                }
                let cloned_node = node.try_clone().map_err(|err| BroadcastingError::ReadNodeError(err.to_string()))?;
                if let Err(BroadcastingError::CanNotRead(err)) = recieve_new_block(log_sender.clone(), cloned_node, blocks.clone()) {
                    write_in_log(log_sender.error_log_sender.clone(), format!("Error al recibir bloque -{:?}- del nodo -{:?}-. Error: {err}", header.hash(), node.peer_addr()).as_str());
                    continue;
                }
            }
        }
    }
    Ok(())
}



fn recieve_new_block(log_sender: LogSender, mut node: TcpStream, blocks: Arc<RwLock<Vec<Block>>>) -> BroadcastingResult {
    let new_block: Block = match BlockMessage::read_from(log_sender.clone(), &mut node)
    {
        Err(err) => {
            return Err(BroadcastingError::CanNotRead(err.to_string()))
        }
        Ok(block) => block,
    };
    if new_block.validate().0 {
        blocks
            .write()
            .map_err(|err| BroadcastingError::LockError(err.to_string()))?
            .push(new_block);
        write_in_log(
            log_sender.info_log_sender.clone(),
            "NUEVO BLOQUE AGREGADO!",
        );
    } else {
        write_in_log(
            log_sender.error_log_sender.clone(),
            "NUEVO BLOQUE ES INVALIDO, NO LO AGREGO!",
        );
    }
    Ok(())
}

fn ask_for_new_block(log_sender: LogSender, mut node:TcpStream, header: BlockHeader) -> BroadcastingResult {
    GetDataMessage::new(vec![Inventory::new_block(header.hash())]).write_to(&mut node).map_err(|err| {
        write_in_log(
            log_sender.error_log_sender.clone(),
            format!(
                "Error al pedir bloque -{:?}- a nodo -{:?}-. Error: {err}",
                header.hash(),
                node.peer_addr()
            )
            .as_str(),
        );
        BroadcastingError::WriteNodeError(err.to_string())
    })?;
    Ok(())              
}
 
fn recieve_new_header(log_sender: LogSender, header: BlockHeader, headers: Arc<RwLock<Vec<BlockHeader>>>) -> BroadcastingResult {
    println!("%%%%%%%    Recibo nuevo header!!!    %%%%%%%");
    headers
        .write()
        .map_err(|err| BroadcastingError::LockError(err.to_string()))?
        .push(header);
    write_in_log(
        log_sender.info_log_sender.clone(),
        "Recibo un nuevo header, lo agrego a la cadena de headers!",
    );
    Ok(())
}


fn get_last_header(headers: Arc<RwLock<Vec<BlockHeader>>>) -> Result<BlockHeader, BroadcastingError> {
    let last_header = *headers
    .read()
    .map_err(|err| BroadcastingError::LockError(err.to_string()))?
    .last()
    .ok_or("No se pudo obtener el último header")
    .map_err(|err| BroadcastingError::CanNotRead(err.to_string()))?;
    Ok(last_header)
}