use crate::{
    blocks::{block::Block, block_header::BlockHeader},
    listener::listen_for_incoming_messages,
    logwriter::log_writer::{write_in_log, LogSender},
    messages::{
        block_message::BlockMessage, get_data_message::GetDataMessage,
        headers_message::is_terminated, inventory::Inventory,
    }, wallet::Wallet, transactions::transaction::Transaction,
};
use std::{
    error::Error,
    fmt,
    net::TcpStream,
    sync::{Arc, Mutex, RwLock},
    thread::{self, JoinHandle}
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
        wallet: Wallet,
    ) -> Result<Self, BroadcastingError> {
        write_in_log(
            log_sender.info_log_sender.clone(),
            "Empiezo a escuchar por nuevos bloques",
        );
        let finish = Arc::new(RwLock::new(false));
        let mut nodes_handle: Vec<JoinHandle<BroadcastingResult>> = vec![];
        let cant_nodos = get_amount_of_nodes(wallet.node.connected_nodes.clone())?;
        // lista de transacciones recibidas para no recibir las mismas de varios nodos
        let transactions_recieved: Arc<RwLock<Vec<[u8; 32]>>> = Arc::new(RwLock::new(Vec::new()));
        let pending_transactions: Arc<RwLock<Vec<Transaction>>> = Arc::new(RwLock::new(Vec::new()));
        let confirmed_transactions: Arc<RwLock<Vec<Transaction>>> = Arc::new(RwLock::new(Vec::new()));
        for _ in 0..cant_nodos {
            let node = get_last_node(wallet.node.connected_nodes.clone())?;
            println!(
                "Nodo -{:?}- Escuchando por nuevos bloques...\n",
                node.peer_addr()
            );
            nodes_handle.push(listen_for_incoming_blocks_from_node(
                log_sender.clone(),
                wallet.clone(),
                transactions_recieved.clone(),
                pending_transactions.clone(),
                confirmed_transactions.clone(),
                node,
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
    wallet: Wallet,
    transactions_recieved: Arc<RwLock<Vec<[u8; 32]>>>,
    pending_transactions: Arc<RwLock<Vec<Transaction>>>,
    confirmed_transactions: Arc<RwLock<Vec<Transaction>>>,
    mut node: TcpStream,
    finish: Arc<RwLock<bool>>,
) -> JoinHandle<BroadcastingResult> {
    let log_sender_clone = log_sender.clone();
    thread::spawn(move || -> BroadcastingResult {
        while !is_terminated(Some(finish.clone())) {
            //listen_for_incoming_messages(log_sender.clone(), &mut node, Some(finish.clone())).map_err(|err| BroadcastingError::ReadNodeError(err.to_string()))?;

            let new_headers = match listen_for_incoming_messages(
                log_sender_clone.clone(),
                wallet.clone(),
                transactions_recieved.clone(),
                pending_transactions.clone(),
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
            let cloned_node = node
                .try_clone()
                .map_err(|err| BroadcastingError::ReadNodeError(err.to_string()))?;
            ask_for_new_blocks(
                log_sender.clone(),
                new_headers,
                cloned_node,
                wallet.clone(), 
                pending_transactions.clone(),
                confirmed_transactions.clone()
            )?;
        }
        Ok(())
    })
}

fn ask_for_new_blocks(
    log_sender: LogSender,
    new_headers: Vec<BlockHeader>,
    node: TcpStream,
    wallet: Wallet, 
    pending_transactions: Arc<RwLock<Vec<Transaction>>>,
    confirmed_transactions: Arc<RwLock<Vec<Transaction>>>,
) -> BroadcastingResult {
    for header in new_headers {
        if !header.validate() {
            write_in_log(
                log_sender.error_log_sender.clone(),
                "Error en validacion de la proof of work de nuevo header",
            );
        } else {
            let last_header = get_last_header(wallet.node.headers.clone())?;
            if last_header != header {
                recieve_new_header(log_sender.clone(), header, wallet.node.headers.clone())?;
                let cloned_node = node
                    .try_clone()
                    .map_err(|err| BroadcastingError::ReadNodeError(err.to_string()))?;
                if ask_for_new_block(log_sender.clone(), cloned_node, header).is_err() {
                    continue;
                }
                let cloned_node = node
                    .try_clone()
                    .map_err(|err| BroadcastingError::ReadNodeError(err.to_string()))?;
                if let Err(BroadcastingError::CanNotRead(err)) =
                    recieve_new_block(log_sender.clone(), cloned_node, wallet.node.block_chain.clone(), pending_transactions.clone(), confirmed_transactions.clone())
                {
                    write_in_log(
                        log_sender.error_log_sender.clone(),
                        format!(
                            "Error al recibir bloque -{:?}- del nodo -{:?}-. Error: {err}",
                            header.hash(),
                            node.peer_addr()
                        )
                        .as_str(),
                    );
                    continue;
                }
            }
        }
    }
    Ok(())
}

fn recieve_new_block(
    log_sender: LogSender,
    mut node: TcpStream,
    blocks: Arc<RwLock<Vec<Block>>>,
    pending_transactions: Arc<RwLock<Vec<Transaction>>>,
    confirmed_transactions: Arc<RwLock<Vec<Transaction>>>
) -> BroadcastingResult {
    let new_block: Block = match BlockMessage::read_from(log_sender.clone(), &mut node) {
        Err(err) => return Err(BroadcastingError::CanNotRead(err.to_string())),
        Ok(block) => block,
    };
    if new_block.validate().0 {
        //new_block.set_utxos(); // seteo utxos de las transacciones del bloque
        
        blocks
            .write()
            .map_err(|err| BroadcastingError::LockError(err.to_string()))?
            .push(new_block.clone());
        write_in_log(log_sender.info_log_sender, "NUEVO BLOQUE AGREGADO!");
        check_if_new_block_contains_pending_tx(new_block, pending_transactions, confirmed_transactions)?;
    } else {
        write_in_log(
            log_sender.error_log_sender,
            "NUEVO BLOQUE ES INVALIDO, NO LO AGREGO!",
        );
    }
    Ok(())
}

fn ask_for_new_block(
    log_sender: LogSender,
    mut node: TcpStream,
    header: BlockHeader,
) -> BroadcastingResult {
    GetDataMessage::new(vec![Inventory::new_block(header.hash())])
        .write_to(&mut node)
        .map_err(|err| {
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

fn recieve_new_header(
    log_sender: LogSender,
    header: BlockHeader,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
) -> BroadcastingResult {
    println!("%%%%%%%    Recibo nuevo header!!!    %%%%%%%");
    headers
        .write()
        .map_err(|err| BroadcastingError::LockError(err.to_string()))?
        .push(header);
    write_in_log(
        log_sender.info_log_sender,
        "Recibo un nuevo header, lo agrego a la cadena de headers!",
    );
    Ok(())
}

fn get_last_header(
    headers: Arc<RwLock<Vec<BlockHeader>>>,
) -> Result<BlockHeader, BroadcastingError> {
    let last_header = *headers
        .read()
        .map_err(|err| BroadcastingError::LockError(err.to_string()))?
        .last()
        .ok_or("No se pudo obtener el último header")
        .map_err(|err| BroadcastingError::CanNotRead(err.to_string()))?;
    Ok(last_header)
}

fn get_last_node(nodes: Arc<RwLock<Vec<TcpStream>>>) -> Result<TcpStream, BroadcastingError> {
    let node = nodes
        .try_write()
        .map_err(|err| BroadcastingError::LockError(err.to_string()))?
        .pop()
        .ok_or("Error no hay mas nodos para descargar los headers!\n")
        .map_err(|err| BroadcastingError::CanNotRead(err.to_string()))?;
    Ok(node)
}

fn get_amount_of_nodes(nodes: Arc<RwLock<Vec<TcpStream>>>) -> Result<usize, BroadcastingError> {
    let amount_of_nodes = nodes
        .read()
        .map_err(|err| BroadcastingError::LockError(err.to_string()))?
        .len();
    Ok(amount_of_nodes)
}

fn check_if_new_block_contains_pending_tx(block: Block, pending_transactions: Arc<RwLock<Vec<Transaction>>>, confirmed_transactions: Arc<RwLock<Vec<Transaction>>>) -> BroadcastingResult {
    for tx in block.txn {
        if pending_transactions.read().map_err(|err| BroadcastingError::LockError(err.to_string()))?.contains(&tx) {
            println!("%%%%%%%%% El bloque contiene la transaccion {:?} confirmada %%%%%%%%%%%", tx.hash());
            let pending_transaction_index = pending_transactions.read().map_err(|err| BroadcastingError::LockError(err.to_string()))?.iter().position(|pending_tx| pending_tx.hash() == tx.hash());
            if let Some(pending_transaction_index) = pending_transaction_index {
                let confirmed_tx = pending_transactions.write().map_err(|err| BroadcastingError::LockError(err.to_string()))?.remove(pending_transaction_index);
                confirmed_transactions.write().map_err(|err| BroadcastingError::LockError(err.to_string()))?.push(confirmed_tx);
            }
        }
    }
    Ok(())
}