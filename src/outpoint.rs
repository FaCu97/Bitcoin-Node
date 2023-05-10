#[derive(PartialEq, Debug)]
pub struct Outpoint {
    tx_id: [u8; 32],
    index: u32,
}

impl Outpoint {
    pub fn new(tx_id: [u8; 32], index: u32) -> Self {
        Outpoint { tx_id, index }
    }

    pub fn unmarshaling(bytes: &[u8]) -> Outpoint {
        let mut offset: usize = 0;
        let mut tx_id: [u8; 32] = [0; 32];
        tx_id[..32].copy_from_slice(&bytes[..32]);
        offset += 32;
        let mut index_bytes: [u8; 4] = [0; 4];
        index_bytes[..4].copy_from_slice(&bytes[offset..(4 + offset)]);
        let index = u32::from_le_bytes(index_bytes);
        Outpoint { tx_id, index }
    }
    // esta funcion se encarga de serializar un outpoint y cargarlo en el array bytes
    pub fn marshaling(&self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(&self.tx_id[0..32]); // se cargan los elementos del tx_id
        let index_bytes: [u8; 4] = self.index.to_le_bytes();
        for i in 0..4 {
            bytes.push(index_bytes[i]);
        }
    }
}

#[cfg(test)]

mod test {
    use super::Outpoint;

    #[test]
    fn test_unmarshaling_del_outpoint_produce_tx_id_esperado() {
        let bytes: Vec<u8> = vec![1; 36];
        let tx_id_esperado: [u8; 32] = [1; 32];
        let outpoint: Outpoint = Outpoint::unmarshaling(&bytes);
        assert_eq!(outpoint.tx_id, tx_id_esperado);
    }

    #[test]
    fn test_unmarshaling_del_outpoint_produce_index_esperado() {
        let mut bytes: Vec<u8> = vec![0; 36];
        for x in 0..4 {
            bytes[32 + x] = x as u8;
        }
        let index_esperado: u32 = 0x03020100;
        let outpoint: Outpoint = Outpoint::unmarshaling(&bytes);
        assert_eq!(outpoint.index, index_esperado);
    }

    #[test]
    fn test_marshaling_del_outpoint_produce_tx_id_esperado() {
        let mut marshaling_outpoint: Vec<u8> = Vec::new();
        let tx_id: [u8; 32] = [2; 32];
        let outpoint_to_marshaling: Outpoint = Outpoint {
            tx_id,
            index: 0x03020100,
        };
        outpoint_to_marshaling.marshaling(&mut marshaling_outpoint);
        let outpoint_unmarshaled: Outpoint = Outpoint::unmarshaling(&marshaling_outpoint);
        assert_eq!(outpoint_unmarshaled.tx_id, tx_id);
    }

    #[test]
    fn test_marshaling_del_outpoint_produce_index_esperado() {
        let mut marshaling_outpoint: Vec<u8> = Vec::new();
        let tx_id: [u8; 32] = [2; 32];
        let index: u32 = 0x03020100;
        let outpoint_to_marshaling: Outpoint = Outpoint { tx_id, index };
        outpoint_to_marshaling.marshaling(&mut marshaling_outpoint);
        let outpoint_unmarshaled: Outpoint = Outpoint::unmarshaling(&marshaling_outpoint);
        assert_eq!(outpoint_unmarshaled.index, index);
    }
}
