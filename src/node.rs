use std::{
    collections::HashMap,
    error::Error,
    io,
    net::TcpStream,
    sync::{Arc, RwLock},
};

use crate::{
    account::Account,
    blocks::{block::Block, block_header::BlockHeader},
    handler::node_message_handler::{NodeMessageHandler, NodeMessageHandlerError},
    logwriter::log_writer::LogSender,
    messages::inventory::{inv_mershalling, Inventory},
    utxo_tuple::UtxoTuple,
};

type UtxoSetPointer = Arc<RwLock<HashMap<[u8; 32], UtxoTuple>>>;

#[derive(Debug, Clone)]
pub struct Node {
    pub connected_nodes: Arc<RwLock<Vec<TcpStream>>>,
    pub headers: Arc<RwLock<Vec<BlockHeader>>>,
    pub block_chain: Arc<RwLock<Vec<Block>>>,
    pub utxo_set: UtxoSetPointer,
    pub accounts: Arc<RwLock<Arc<RwLock<Vec<Account>>>>>,
    pub peers_handler: NodeMessageHandler,
}

impl Node {
    /// Inicializa el nodo. Recibe la blockchain ya descargada.
    pub fn new(
        log_sender: LogSender,
        connected_nodes: Arc<RwLock<Vec<TcpStream>>>,
        headers: Arc<RwLock<Vec<BlockHeader>>>,
        block_chain: Arc<RwLock<Vec<Block>>>,
    ) -> Result<Self, NodeMessageHandlerError> {
        let pointer_to_utxo_set: UtxoSetPointer = Arc::new(RwLock::new(HashMap::new()));
        generate_utxo_set(&block_chain, pointer_to_utxo_set.clone())?;
        let pointer_to_accounts_in_node = Arc::new(RwLock::new(Arc::new(RwLock::new(vec![]))));
        let peers_handler = NodeMessageHandler::new(
            log_sender,
            headers.clone(),
            block_chain.clone(),
            connected_nodes.clone(),
            pointer_to_accounts_in_node.clone(),
            pointer_to_utxo_set.clone(),
        )?;
        Ok(Node {
            connected_nodes,
            headers,
            block_chain,
            utxo_set: pointer_to_utxo_set,
            accounts: pointer_to_accounts_in_node,
            peers_handler,
        })
    }
    /// Validar el bloque recibido
    pub fn block_validation(block: Block) -> (bool, &'static str) {
        block.validate()
    }

    /// Devuelve las utxos asociadas a la address recibida.
    pub fn utxos_referenced_to_account(
        &self,
        address: &str,
    ) -> Result<Vec<UtxoTuple>, Box<dyn Error>> {
        let mut account_utxo_set: Vec<UtxoTuple> = Vec::new();
        for utxo in self
            .utxo_set
            .read()
            .map_err(|err| NodeMessageHandlerError::LockError(err.to_string()))?
            .values()
        {
            let aux_utxo = utxo.referenced_utxos(address);
            let utxo_to_push = match aux_utxo {
                Some(value) => value,
                None => continue,
            };
            account_utxo_set.push(utxo_to_push);
        }
        Ok(account_utxo_set)
    }
    /// Se encarga de llamar a la funcion finish() del peers_handler del nodo
    pub fn shutdown_node(&self) -> Result<(), NodeMessageHandlerError> {
        self.peers_handler.finish()
    }

    /// Recibe un vector de bytes que representa a la raw format transaction para se enviada por
    /// la red a todos los nodos conectados
    pub fn broadcast_tx(&self, raw_tx: [u8; 32]) -> Result<(), NodeMessageHandlerError> {
        let inventories = vec![Inventory::new_tx(raw_tx)];
        let inv_message_bytes = inv_mershalling(inventories);
        self.peers_handler.broadcast_to_nodes(inv_message_bytes)
    }

    /// Actualiza lo que apunta el puntero de accounts a otro puntero que es pasado por parametro
    /// de esta manera el puntero queda apuntando a un puntero con un vector de cuentas que es apuntado por la wallet
    pub fn set_accounts(
        &mut self,
        accounts: Arc<RwLock<Vec<Account>>>,
    ) -> Result<(), NodeMessageHandlerError> {
        *self
            .accounts
            .write()
            .map_err(|err| NodeMessageHandlerError::LockError(err.to_string()))? = accounts;
        Ok(())
    }

    ///
    pub fn merkle_proof_of_inclusion(
        &self,
        block_hash: &[u8; 32],
        tx_hash: &[u8; 32],
    ) -> Result<Option<Vec<([u8; 32], bool)>>, NodeMessageHandlerError> {
        let block_chain = self
            .block_chain
            .write()
            .map_err(|err| NodeMessageHandlerError::LockError(err.to_string()))?;
        let mut index = block_chain.len() - 1;
        while index > 0 {
            if block_chain[index].is_same_block(block_hash) {
                return Ok(block_chain[index].merkle_proof_of_inclusion(tx_hash));
            }
            index -= 1;
        }
        return Err(Box::new(std::io::Error::new(
            io::ErrorKind::Other,
            "No se encontró el bloque",
        )))
        .map_err(|err| NodeMessageHandlerError::LockError(err.to_string()))?;
    }
}

/// Funcion que se encarga de generar la lista de utxos
fn generate_utxo_set(
    block_chain: &Arc<RwLock<Vec<Block>>>,
    utxo_set: UtxoSetPointer,
) -> Result<UtxoSetPointer, NodeMessageHandlerError> {
    let block_chain_lock = block_chain
        .read()
        .map_err(|err| NodeMessageHandlerError::LockError(err.to_string()))?;

    for block in block_chain_lock.iter() {
        block
            .give_me_utxos(utxo_set.clone())
            .map_err(|err| NodeMessageHandlerError::LockError(err.to_string()))?;
    }
    Ok(utxo_set)
}
