use crate::compact_size_uint::CompactSizeUint;

use super::inventory::Inventory;

/// Representa el mensaje Inv del protocolo bitcoin.
/// Transmite uno o varios inventories (hashes).
/// Puede ser la respuesta al mensaje getdata
#[derive(Debug)]
pub struct GetDataPayload {
    count: CompactSizeUint,
    inventories: Vec<Inventory>,
    get_data_payload_bytes: Vec<u8>,
}

impl GetDataPayload {
    /// Dado un vector de inventory, devuelve el payload del mensaje getdata
    pub fn get_payload(inventories: Vec<Inventory>) -> GetDataPayload {
        let count = CompactSizeUint::new(inventories.len() as u128);
        let get_data_payload_bytes = get_data_payload_bytes(&count, &inventories);
        GetDataPayload {
            count,
            inventories,
            get_data_payload_bytes,
        }
    }

    /// Devuelve un vector de bytes que representan el payload del mensaje getdata
    pub fn to_le_bytes(&self) -> &[u8] {
        &self.get_data_payload_bytes
    }

    /// Devuelve el tamaño en bytes del payload
    pub fn size(&self) -> usize {
        self.to_le_bytes().len()
    }
}

/// Devuelve el payload serializado a bytes
fn get_data_payload_bytes(count: &CompactSizeUint, inventories: &Vec<Inventory>) -> Vec<u8> {
    let mut getdata_payload_bytes: Vec<u8> = vec![];
    getdata_payload_bytes.extend_from_slice(&count.marshalling());
    for inventory in inventories {
        getdata_payload_bytes.extend(inventory.to_le_bytes());
    }
    getdata_payload_bytes
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_con_un_inventory_se_crea_correctamente() {
        // GIVEN : un inventory con un solo hash
        let mut inventories = Vec::new();
        inventories.push(Inventory::new_block([0; 32]));
        // WHEN: se llama al método get_payload
        let payload = GetDataPayload::get_payload(inventories.clone());
        // THEN: los atributos del GetDataPayload se crearon correctamente.
        assert_eq!(payload.count.decoded_value() as usize, inventories.len());
        //assert_eq!(payload.inventories, inventories);
    }
    #[test]
    fn payload_con_dos_inventory_se_crea_correctamente() {
        // GIVEN : un inventory con un solo hash
        let mut inventories = Vec::new();
        inventories.push(Inventory::new_block([0; 32]));
        inventories.push(Inventory::new_block([0; 32]));
        // WHEN: se llama al método get_payload
        let payload = GetDataPayload::get_payload(inventories.clone());
        // THEN: los atributos del GetDataPayload se crearon correctamente.
        assert_eq!(payload.count.decoded_value() as usize, inventories.len());
       // assert_eq!(payload.inventories, inventories);
    }
}
