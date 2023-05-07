use bitcoin_hashes::{sha256, Hash};
#[derive(Debug)]
pub struct BlockHeader {
    pub version: i32,
    pub previous_block_header_hash: [u8; 32],
    pub merkle_root_hash: [u8; 32],
    pub time: u32,
    pub n_bits: u32,
    pub nonce: u32,
    pub hash: [u8; 32],
}

impl BlockHeader {
    pub fn unmarshaling(block_header_message: [u8; 80]) -> BlockHeader {
        let mut offset: usize = 0;
        let mut version_bytes: [u8; 4] = [0; 4];
        Self::read_bytes(&block_header_message, 4, &mut version_bytes, offset);
        offset += 4;
        let version = i32::from_le_bytes(version_bytes);
        let mut previous_block_header_hash: [u8; 32] = [0; 32];
        Self::read_bytes(
            &block_header_message,
            32,
            &mut previous_block_header_hash,
            offset,
        );
        offset += 32;
        let mut merkle_root_hash: [u8; 32] = [0; 32];
        Self::read_bytes(&block_header_message, 32, &mut merkle_root_hash, offset);
        offset += 32;
        let mut time_bytes: [u8; 4] = [0; 4];
        Self::read_bytes(&block_header_message, 4, &mut time_bytes, offset);
        offset += 4;
        let time = u32::from_le_bytes(time_bytes);
        let mut n_bits_bytes: [u8; 4] = [0; 4];
        Self::read_bytes(&block_header_message, 4, &mut n_bits_bytes, offset);
        offset += 4;
        let n_bits = u32::from_le_bytes(n_bits_bytes);
        let mut nonce_bytes: [u8; 4] = [0; 4];
        Self::read_bytes(&block_header_message, 4, &mut nonce_bytes, offset);
        let nonce = u32::from_le_bytes(nonce_bytes);
        BlockHeader {
            version,
            previous_block_header_hash,
            merkle_root_hash,
            time,
            n_bits,
            nonce,
            hash: [0; 32],
        }
    }
    fn read_bytes(block_header_message: &[u8], amount: usize, aux_bytes: &mut [u8], offset: usize) {
        for byte in 0..amount {
            aux_bytes[byte] = block_header_message[byte + offset];
        }
    }
    fn write_bytes(
        block_header_message: &mut [u8],
        amount: usize,
        aux_bytes: &[u8],
        offset: usize,
    ) {
        for byte in 0..amount {
            block_header_message[byte + offset] = aux_bytes[byte];
        }
    }
    pub fn marshaling(&self, marshaled_block_header: &mut [u8]) {
        let mut offset = 0;
        let version_bytes = self.version.to_le_bytes();
        Self::write_bytes(marshaled_block_header, 4, &version_bytes, offset);
        offset += 4;
        Self::write_bytes(
            marshaled_block_header,
            32,
            &self.previous_block_header_hash,
            offset,
        );
        offset += 32;
        Self::write_bytes(marshaled_block_header, 32, &self.merkle_root_hash, offset);
        offset += 32;
        let time_bytes = self.time.to_le_bytes();
        Self::write_bytes(marshaled_block_header, 4, &time_bytes, offset);
        offset += 4;
        let n_bits_bytes = self.n_bits.to_le_bytes();
        Self::write_bytes(marshaled_block_header, 4, &n_bits_bytes, offset);
        offset += 4;
        let nonce_bytes = self.nonce.to_le_bytes();
        Self::write_bytes(marshaled_block_header, 4, &nonce_bytes, offset);
    }
    pub fn hash(&mut self) {
        let mut block_header_marshaled: [u8; 80] = [0; 80];
        self.marshaling(&mut block_header_marshaled);
        let hash_block = sha256::Hash::hash(&block_header_marshaled);
        self.hash = *hash_block.as_byte_array();
    }
}

#[cfg(test)]
mod tests {
    use crate::block_header::BlockHeader;
    use bitcoin_hashes::{sha256, Hash};

    #[test]
    fn test_deserializacion_del_header_genera_version_esperada() {
        let mut message_header = [0; 80];
        for i in 0..80 {
            message_header[i] = i as u8;
        }
        let blockeheader = BlockHeader::unmarshaling(message_header);
        let expected_value = 0x3020100;
        assert_eq!(blockeheader.version, expected_value);
    }

    #[test]
    fn test_deserializacion_del_header_genera_previous_block_header_hash_esperado() {
        let mut message_header = [0; 80];
        for i in 0..80 {
            message_header[i] = i as u8;
        }
        let blockeheader = BlockHeader::unmarshaling(message_header);
        let expected_value = [
            4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
            27, 28, 29, 30, 31, 32, 33, 34, 35,
        ];
        assert_eq!(blockeheader.previous_block_header_hash, expected_value);
    }

    #[test]
    fn test_deserializacion_del_header_genera_merkle_root_hash_esperado() {
        let mut message_header = [0; 80];
        for i in 0..80 {
            message_header[i] = i as u8;
        }
        let blockeheader = BlockHeader::unmarshaling(message_header);
        let expected_value = [
            36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57,
            58, 59, 60, 61, 62, 63, 64, 65, 66, 67,
        ];
        assert_eq!(blockeheader.merkle_root_hash, expected_value);
    }

    #[test]
    fn test_deserializacion_del_header_genera_time_esperado() {
        let mut message_header = [0; 80];
        for i in 0..80 {
            message_header[i] = i as u8;
        }
        let blockeheader = BlockHeader::unmarshaling(message_header);
        let expected_value = 0x47464544;
        assert_eq!(blockeheader.time, expected_value);
    }

    #[test]
    fn test_deserializacion_del_header_genera_nbits_esperado() {
        let mut message_header = [0; 80];
        for i in 0..80 {
            message_header[i] = i as u8;
        }
        let blockeheader = BlockHeader::unmarshaling(message_header);
        let expected_value = 0x4B4A4948;
        assert_eq!(blockeheader.n_bits, expected_value);
    }

    #[test]
    fn test_deserializacion_del_header_genera_nonce_esperado() {
        let mut message_header = [0; 80];
        for i in 0..80 {
            message_header[i] = i as u8;
        }
        let blockeheader = BlockHeader::unmarshaling(message_header);
        let expected_value = 0x4F4E4D4C;
        assert_eq!(blockeheader.nonce, expected_value);
    }

    #[test]
    fn test_serializacion_correcta_del_campo_version() {
        let mut block_header_message: [u8; 80] = [0; 80];
        let block = BlockHeader {
            version: 50462976,
            previous_block_header_hash: [0; 32],
            merkle_root_hash: [0; 32],
            time: 0,
            n_bits: 0,
            nonce: 0,
            hash: [0; 32],
        };
        block.marshaling(&mut block_header_message);
        let expected_block = BlockHeader::unmarshaling(block_header_message);
        let expected_value = 0x3020100;
        assert_eq!(expected_block.version, expected_value);
    }
    #[test]
    fn test_serializacion_correcta_del_campo_previous_block_header_hash() {
        let mut block_header_message: [u8; 80] = [0; 80];
        let value = [1; 32];
        let block = BlockHeader {
            version: 0,
            previous_block_header_hash: value,
            merkle_root_hash: [0; 32],
            time: 0,
            n_bits: 0,
            nonce: 0,
            hash: [0; 32],
        };
        block.marshaling(&mut block_header_message);
        let expected_block = BlockHeader::unmarshaling(block_header_message);
        assert_eq!(expected_block.previous_block_header_hash, value);
    }
    #[test]
    fn test_serializacion_correcta_del_campo_merkle_root_hash() {
        let mut block_header_message: [u8; 80] = [0; 80];
        let value = [1; 32];
        let block = BlockHeader {
            version: 0,
            previous_block_header_hash: [0; 32],
            merkle_root_hash: value,
            time: 0,
            n_bits: 0,
            nonce: 0,
            hash: [0; 32],
        };
        block.marshaling(&mut block_header_message);
        let expected_block = BlockHeader::unmarshaling(block_header_message);
        assert_eq!(expected_block.merkle_root_hash, value);
    }
    #[test]
    fn test_serializacion_correcta_del_campo_time() {
        let mut block_header_message: [u8; 80] = [0; 80];
        let value = 0x03020100;
        let block = BlockHeader {
            version: 0,
            previous_block_header_hash: [0; 32],
            merkle_root_hash: [0; 32],
            time: value,
            n_bits: 0,
            nonce: 0,
            hash: [0; 32],
        };
        block.marshaling(&mut block_header_message);
        let expected_block = BlockHeader::unmarshaling(block_header_message);
        assert_eq!(expected_block.time, value);
    }
    #[test]
    fn test_serializacion_correcta_del_campo_nbits() {
        let mut block_header_message: [u8; 80] = [0; 80];
        let value = 0x03020100;
        let block = BlockHeader {
            version: 0,
            previous_block_header_hash: [0; 32],
            merkle_root_hash: [0; 32],
            time: 0,
            n_bits: value,
            nonce: 0,
            hash: [0; 32],
        };
        block.marshaling(&mut block_header_message);
        let expected_block = BlockHeader::unmarshaling(block_header_message);
        assert_eq!(expected_block.n_bits, value);
    }
    #[test]
    fn test_serializacion_correcta_del_campo_nonce() {
        let mut block_header_message: [u8; 80] = [0; 80];
        let value = 0x03020100;
        let block = BlockHeader {
            version: 0,
            previous_block_header_hash: [0; 32],
            merkle_root_hash: [0; 32],
            time: 0,
            n_bits: 0,
            nonce: value,
            hash: [0; 32],
        };
        block.marshaling(&mut block_header_message);
        let expected_block = BlockHeader::unmarshaling(block_header_message);
        assert_eq!(expected_block.nonce, value);
    }
    #[test]
    fn test_el_header_es_hasheado_correctamente() {
        let mut block_header = BlockHeader {
            version: 0x03020100,
            previous_block_header_hash: [0; 32],
            merkle_root_hash: [0; 32],
            time: 0,
            n_bits: 0,
            nonce: 0,
            hash: [0; 32],
        };
        let mut block_header_message_expected: [u8; 80] = [0; 80];
        for x in 0..4 {
            block_header_message_expected[x] = x as u8;
        }
        let expected_hash = sha256::Hash::hash(&block_header_message_expected);
        block_header.hash();
        assert_eq!(block_header.hash, *expected_hash.as_byte_array())
    }
}
