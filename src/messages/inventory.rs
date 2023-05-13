#[derive(Debug, Clone)]
pub struct Inventory {
    type_identifier: u32,
    hash: [u8;32]
}

impl Inventory {
    pub fn new_block(hash: [u8;32]) -> Inventory {
        Inventory{
            type_identifier: 2, // 2: Block
            hash,
        }
    }
    
    pub fn to_le_bytes(&self) -> Vec<u8> {
        let mut inventory_bytes: Vec<u8> = Vec::new();
        inventory_bytes.extend_from_slice(&self.type_identifier.to_le_bytes());
        inventory_bytes.extend(self.hash);
        inventory_bytes
    }

    pub fn from_le_bytes(inventory_bytes: &[u8]) -> Inventory{
        let mut type_identifier_bytes = [0;4];
        type_identifier_bytes.copy_from_slice(&inventory_bytes[0..4]);
        let mut hash_bytes = [0;32];
        hash_bytes.copy_from_slice(&inventory_bytes[4..36]);
        Inventory {
            type_identifier: u32::from_le_bytes(type_identifier_bytes),
            hash: hash_bytes,
        }
    }
}