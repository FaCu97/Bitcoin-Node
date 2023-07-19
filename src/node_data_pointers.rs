use std::{
    collections::HashMap,
    net::TcpStream,
    sync::{Arc, RwLock},
};

use crate::{
    account::Account,
    blocks::{block::Block, block_header::BlockHeader},
    utxo_tuple::UtxoTuple,
};

type UtxoSetPointer = Arc<RwLock<HashMap<[u8; 32], UtxoTuple>>>;

/// Almacena los punteros de los datos del nodo que se comparten entre los hilos.
#[derive(Debug, Clone)]
pub struct NodeDataPointers {
    pub connected_nodes: Arc<RwLock<Vec<TcpStream>>>,
    pub headers: Arc<RwLock<Vec<BlockHeader>>>,
    pub block_chain: Arc<RwLock<HashMap<[u8; 32], Block>>>,
    pub header_heights: Arc<RwLock<HashMap<[u8; 32], usize>>>,
    pub accounts: Arc<RwLock<Arc<RwLock<Vec<Account>>>>>,
    pub utxo_set: UtxoSetPointer,
}

impl NodeDataPointers {
    /// Almacena los punteros de los datos del nodo que se comparten entre los hilos.
    pub fn new(
        connected_nodes: Arc<RwLock<Vec<TcpStream>>>,
        headers: Arc<RwLock<Vec<BlockHeader>>>,
        block_chain: Arc<RwLock<HashMap<[u8; 32], Block>>>,
        header_heights: Arc<RwLock<HashMap<[u8; 32], usize>>>,
        accounts: Arc<RwLock<Arc<RwLock<Vec<Account>>>>>,
        utxo_set: UtxoSetPointer,
    ) -> Self {
        NodeDataPointers {
            connected_nodes,
            headers,
            block_chain,
            header_heights,
            accounts,
            utxo_set,
        }
    }
}
