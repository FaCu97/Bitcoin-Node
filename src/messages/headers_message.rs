use crate::block_header::BlockHeader;
use crate::compact_size_uint::CompactSizeUint;
const BLOCK_HEADER_SIZE: usize = 80;
pub struct HeadersMessage;

impl HeadersMessage {
    /// Recibe en bytes la respuesta del mensaje headers.
    /// Devuelve un vector con los block headers contenidos
    pub fn unmarshaling(headers_message_bytes: &Vec<u8>) -> Result<Vec<BlockHeader>, &'static str> {
        let mut block_header_vec = Vec::new();
        let mut offset: usize = 0;
        let count = CompactSizeUint::unmarshaling(headers_message_bytes, &mut offset);
        let headers_size = headers_message_bytes.len();
        let mut i = 0;
        while i < count.decoded_value() {
            let mut header: [u8; BLOCK_HEADER_SIZE] = [0; BLOCK_HEADER_SIZE];
            if offset + BLOCK_HEADER_SIZE > headers_size {
                return Err("Fuera de rango");
            }
            header[0..BLOCK_HEADER_SIZE]
                .copy_from_slice(&headers_message_bytes[(offset)..(offset + BLOCK_HEADER_SIZE)]);
            //el 1 es el transaction_count que viene como 0x00
            offset += BLOCK_HEADER_SIZE + 1;
            i += 1;
            block_header_vec.push(BlockHeader::unmarshaling(header));
        }

        Ok(block_header_vec)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        block_header::BlockHeader, compact_size_uint::CompactSizeUint,
        messages::headers_message::HeadersMessage,
    };

    #[test]
    fn test_deserializacion_del_headers_message_vacio_no_da_block_headers(
    ) -> Result<(), &'static str> {
        // Caso borde, no se si es posible que devuelva 0 block headers.
        let headers_message: Vec<u8> = vec![0; 1];
        let block_headers = HeadersMessage::unmarshaling(&headers_message)?;
        let expected_value = 0;
        assert_eq!(block_headers.len(), expected_value);
        Ok(())
    }

    #[test]
    fn test_deserializacion_del_headers_message_devuelve_1_block_header() -> Result<(), &'static str>
    {
        let headers_message: Vec<u8> = vec![1; 82];
        let block_headers = HeadersMessage::unmarshaling(&headers_message)?;
        let expected_value = 1;
        assert_eq!(block_headers.len(), expected_value);
        Ok(())
    }

    #[test]
    fn test_deserializacion_del_headers_message_devuelve_2_block_header() -> Result<(), &'static str>
    {
        let headers_message: Vec<u8> = vec![2; 163];
        let block_headers = HeadersMessage::unmarshaling(&headers_message)?;
        let expected_value = 2;
        assert_eq!(block_headers.len(), expected_value);
        Ok(())
    }

    #[test]
    fn test_deserializacion_del_headers_message_devuelve_el_block_header_correcto(
    ) -> Result<(), &'static str> {
        let mut headers_message: Vec<u8> = vec![0; 82];
        for i in 1..83 {
            headers_message[i - 1] = i as u8;
        }

        let block_headers = HeadersMessage::unmarshaling(&headers_message)?;

        let mut expected_block_header_bytes: [u8; 80] = [2; 80];
        expected_block_header_bytes.copy_from_slice(&headers_message[1..81]);
        let expected_block_header = BlockHeader::unmarshaling(expected_block_header_bytes);
        let received_block_header = &block_headers[0];

        assert_eq!(received_block_header.version, expected_block_header.version);
        assert_eq!(
            received_block_header.previous_block_header_hash,
            expected_block_header.previous_block_header_hash
        );
        assert_eq!(
            received_block_header.merkle_root_hash,
            expected_block_header.merkle_root_hash
        );
        assert_eq!(received_block_header.time, expected_block_header.time);
        assert_eq!(received_block_header.n_bits, expected_block_header.n_bits);
        assert_eq!(received_block_header.nonce, expected_block_header.nonce);
        assert_eq!(received_block_header.hash, expected_block_header.hash);
        Ok(())
    }

    #[test]
    fn test_deserializacion_del_headers_message_con_515_block_headers() -> Result<(), &'static str>
    {
        let mut headers_message: Vec<u8> = Vec::new();
        let count = CompactSizeUint::new(515);
        headers_message.extend_from_slice(count.value());

        for i in 0..(41718 - 3) {
            headers_message.push(i as u8);
        }
        let block_headers = HeadersMessage::unmarshaling(&headers_message)?;

        let mut expected_block_header_bytes: [u8; 80] = [2; 80];
        expected_block_header_bytes.copy_from_slice(&headers_message[3..83]);
        let expected_block_header = BlockHeader::unmarshaling(expected_block_header_bytes);
        let received_block_header = &block_headers[0];
        let expected_len = 515;

        assert_eq!(block_headers.len(), expected_len);
        assert_eq!(received_block_header.version, expected_block_header.version);
        assert_eq!(
            received_block_header.previous_block_header_hash,
            expected_block_header.previous_block_header_hash
        );
        assert_eq!(
            received_block_header.merkle_root_hash,
            expected_block_header.merkle_root_hash
        );
        assert_eq!(received_block_header.time, expected_block_header.time);
        assert_eq!(received_block_header.n_bits, expected_block_header.n_bits);
        assert_eq!(received_block_header.nonce, expected_block_header.nonce);
        assert_eq!(received_block_header.hash, expected_block_header.hash);
        Ok(())
    }
}
