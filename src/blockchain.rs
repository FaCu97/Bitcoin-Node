use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::blocks::{block::Block, block_header::BlockHeader};
#[derive(Debug, Clone)]

pub struct Blockchain {
    pub headers: Arc<RwLock<Vec<BlockHeader>>>,
    pub blocks: Arc<RwLock<HashMap<[u8; 32], Block>>>,
    pub header_heights: Arc<RwLock<HashMap<[u8; 32], usize>>>,
}

impl Blockchain {
    pub fn new(
        headers: Arc<RwLock<Vec<BlockHeader>>>,
        blocks: Arc<RwLock<HashMap<[u8; 32], Block>>>,
        header_heights: Arc<RwLock<HashMap<[u8; 32], usize>>>,
    ) -> Self {
        Blockchain {
            headers,
            blocks,
            header_heights,
        }
    }
}
