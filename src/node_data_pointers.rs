use std::{
    collections::HashMap,
    net::TcpStream,
    sync::{Arc, RwLock},
};

use crate::{account::Account, blockchain::Blockchain, utxo_tuple::UtxoTuple};

type UtxoSetPointer = Arc<RwLock<HashMap<[u8; 32], UtxoTuple>>>;

/// Almacena los punteros de los datos del nodo que se comparten entre los hilos.
#[derive(Debug, Clone)]
pub struct NodeDataPointers {
    pub connected_nodes: Arc<RwLock<Vec<TcpStream>>>,
    pub blockchain: Blockchain,
    pub accounts: Arc<RwLock<Arc<RwLock<Vec<Account>>>>>,
    pub utxo_set: UtxoSetPointer,
}

impl NodeDataPointers {
    /// Almacena los punteros de los datos del nodo que se comparten entre los hilos.
    pub fn new(
        connected_nodes: Arc<RwLock<Vec<TcpStream>>>,
        blockchain: Blockchain,
        accounts: Arc<RwLock<Arc<RwLock<Vec<Account>>>>>,
        utxo_set: UtxoSetPointer,
    ) -> Self {
        NodeDataPointers {
            connected_nodes,
            blockchain,
            accounts,
            utxo_set,
        }
    }
}
