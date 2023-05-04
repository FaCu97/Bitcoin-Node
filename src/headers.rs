use crate::block_header::BlockHeader;
//use crate::compact_size_uint::CompactSizeUint;
const BLOCK_HEADER_SIZE: usize = 80;
// No se si es necesario guardar en un struct
pub struct Headers {
//    count: CompactSizeUint,
//    headers: Vec<BlockHeader>
}

impl Headers {
    pub fn unmarshaling(headers_message_bytes:&[u8]) -> Vec<BlockHeader> {
        let mut block_header_vec = Vec::new();
        // Falta implementar el compact size
        // Por ahora lo hardcodeo con 1 byte
        let mut count:[u8;1] =[0;1];
        count.copy_from_slice(&headers_message_bytes[0..1]);
        //let count = CompactSizeUint::unmarshal_first(&headers_message_bytes);
        let mut offset = std::mem::size_of_val(&count);
        let headers_size = (*headers_message_bytes).len();
        let mut i = 0;
        while i < count[0] {
            let mut header:[u8;BLOCK_HEADER_SIZE] = [0;BLOCK_HEADER_SIZE];
            header[0..BLOCK_HEADER_SIZE].copy_from_slice(&headers_message_bytes[(offset)..(offset+BLOCK_HEADER_SIZE)]);
            //el 1 es el transaction_count que viene como 0x00
            offset += BLOCK_HEADER_SIZE + 1;
            i+=1;
            block_header_vec.push(BlockHeader::unmarshaling(header));
        }
       
        block_header_vec
    }
}

#[cfg(test)]
mod tests {
    use crate::headers::Headers;

    #[test]
    fn test_deserializacion_del_headers_message_vacio_no_da_block_headers() {
        // Caso borde, no se si es posible que devuelva 0 block headers.
        let headers_message:[u8;1] = [0;1];
        let block_headers = Headers::unmarshaling(&headers_message);
        let expected_value = 0;
        assert_eq!(block_headers.len(), expected_value);
    }

    #[test]
    fn test_deserializacion_del_headers_message_devuelve_1_block_header() {
        // Caso borde, no se si es posible que devuelva 0 block headers.
        let headers_message:[u8;82] = [1;82];
        let block_headers = Headers::unmarshaling(&headers_message);
        let expected_value = 1;
        assert_eq!(block_headers.len(), expected_value);
    }

    #[test]
    fn test_deserializacion_del_headers_message_devuelve_2_block_header() {
        // Caso borde, no se si es posible que devuelva 0 block headers.
        let headers_message:[u8;163] = [2;163];
        let block_headers = Headers::unmarshaling(&headers_message);
        let expected_value = 2;
        assert_eq!(block_headers.len(), expected_value);
    }


}