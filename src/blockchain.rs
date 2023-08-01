use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    blocks::{block::Block, block_header::BlockHeader},
    utxo_tuple::UtxoTuple,
};
type UtxoSetPointer = Arc<RwLock<HashMap<[u8; 32], UtxoTuple>>>;

#[derive(Debug, Clone)]
/// Representa la cadena de bloques con sus bloques, headers, alturas y UTXO set.
pub struct Blockchain {
    pub headers: Arc<RwLock<Vec<BlockHeader>>>,
    pub blocks: Arc<RwLock<HashMap<[u8; 32], Block>>>,
    pub header_heights: Arc<RwLock<HashMap<[u8; 32], usize>>>,
    pub utxo_set: UtxoSetPointer,
}

impl Blockchain {
    /// Crea un nuevo Blockchain que agrupa los headers, bloques, alturas y UTXO set.
    pub fn new(
        headers: Arc<RwLock<Vec<BlockHeader>>>,
        blocks: Arc<RwLock<HashMap<[u8; 32], Block>>>,
        header_heights: Arc<RwLock<HashMap<[u8; 32], usize>>>,
        utxo_set: UtxoSetPointer,
    ) -> Self {
        Blockchain {
            headers,
            blocks,
            header_heights,
            utxo_set,
        }
    }
}
