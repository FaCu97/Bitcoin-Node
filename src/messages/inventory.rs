use crate::compact_size_uint::CompactSizeUint;

#[derive(Debug, Clone)]
pub struct Inventory {
    pub type_identifier: u32,
    pub hash: [u8; 32],
}

impl Inventory {
    pub fn new_block(hash: [u8; 32]) -> Inventory {
        Inventory {
            type_identifier: 2, // 2: Block
            hash,
        }
    }

    pub fn new_tx(hash: [u8; 32]) -> Inventory {
        Inventory {
            type_identifier: 1, // 1: Transaction
            hash,
        }
    }

    pub fn to_le_bytes(&self) -> Vec<u8> {
        let mut inventory_bytes: Vec<u8> = Vec::new();
        inventory_bytes.extend_from_slice(&self.type_identifier.to_le_bytes());
        inventory_bytes.extend(self.hash);
        inventory_bytes
    }

    pub fn from_le_bytes(inventory_bytes: &[u8]) -> Inventory {
        let mut type_identifier_bytes = [0; 4];
        type_identifier_bytes.copy_from_slice(&inventory_bytes[0..4]);
        let mut hash_bytes = [0; 32];
        hash_bytes.copy_from_slice(&inventory_bytes[4..36]);
        Inventory {
            type_identifier: u32::from_le_bytes(type_identifier_bytes),
            hash: hash_bytes,
        }
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
}


pub fn inv_message_bytes(inventories: Vec<Inventory>) -> Vec<u8> {
    let count = CompactSizeUint::new(inventories.len() as u128);
    
}