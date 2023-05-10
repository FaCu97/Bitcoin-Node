
#[derive(PartialEq)]
#[derive(Debug)]
pub struct Outpoint {
    tx_id: [u8; 32],
    index: u32,
}

impl Outpoint {

    pub fn new(tx_id:[u8;32],index:u32)->Self{
        Outpoint { tx_id, index }
    }


    pub fn unmarshalling(bytes: &Vec<u8>,offset : &mut usize) -> Outpoint {
        let mut tx_id: [u8; 32] = [0; 32];
        for x in 0..32 {
            tx_id[x] = bytes[x + (*offset)];
        }
        *offset += 32;
        let mut index_bytes: [u8; 4] = [0; 4];
        for x in 0..4 {
            index_bytes[x] = bytes[x + (*offset)];
        }
        *offset += 4;
        let index = u32::from_le_bytes(index_bytes);
        Outpoint { tx_id, index }
    }
    // esta funcion se encarga de serializar un outpoint y cargarlo en el array bytes
    pub fn marshalling(&self, bytes: &mut Vec<u8>) {
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
    fn test_unmarshalling_del_outpoint_produce_tx_id_esperado() {
        let bytes: Vec<u8> = vec![1; 36];
        let tx_id_esperado: [u8; 32] = [1; 32];
        let mut offset : usize = 0;
        let outpoint: Outpoint = Outpoint::unmarshalling(&bytes,&mut offset);
        assert_eq!(outpoint.tx_id, tx_id_esperado);
    }

    #[test]
    fn test_unmarshalling_del_outpoint_produce_index_esperado() {
        let mut bytes: Vec<u8> = vec![0; 36];
        for x in 0..4 {
            bytes[32 + x] = x as u8;
        }
        let index_esperado: u32 = 0x03020100;
        let mut offset : usize = 0;
        let outpoint: Outpoint = Outpoint::unmarshalling(&bytes,&mut offset);
        assert_eq!(outpoint.index, index_esperado);
    }

    #[test]
    fn test_marshalling_del_outpoint_produce_tx_id_esperado() {
        let mut marshalling_outpoint: Vec<u8> = Vec::new();
        let tx_id: [u8; 32] = [2; 32];
        let outpoint_to_marshalling: Outpoint = Outpoint {
            tx_id,
            index: 0x03020100,
        };
        outpoint_to_marshalling.marshalling(&mut marshalling_outpoint);
        let mut offset : usize = 0;
        let outpoint_unmarshaled: Outpoint = Outpoint::unmarshalling(&marshalling_outpoint,&mut offset);
        assert_eq!(outpoint_unmarshaled.tx_id, tx_id);
    }

    #[test]
    fn test_marshalling_del_outpoint_produce_index_esperado() {
        let mut marshalling_outpoint: Vec<u8> = Vec::new();
        let tx_id: [u8; 32] = [2; 32];
        let index: u32 = 0x03020100;
        let outpoint_to_marshalling: Outpoint = Outpoint { tx_id, index };
        outpoint_to_marshalling.marshalling(&mut marshalling_outpoint);
        let mut offset : usize = 0;
        let outpoint_unmarshaled: Outpoint = Outpoint::unmarshalling(&marshalling_outpoint,&mut offset);
        assert_eq!(outpoint_unmarshaled.index, index);
    }
}
