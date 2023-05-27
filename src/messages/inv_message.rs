use std::{net::TcpStream, io::Read};
use crate::compact_size_uint::CompactSizeUint;
use super::message_header::HeaderMessage;

#[derive(Debug, Clone)]
pub struct InvMessage {
    pub count: CompactSizeUint,
    pub invetories: [u8; 32],
}

impl InvMessage {
    pub fn read_inv_payload_from(header: HeaderMessage, stream: &mut TcpStream) {
        let payload_size: usize = header.payload_size as usize;
        let mut payload: Vec<u8> = vec![0; payload_size];
        stream.read_exact(&mut payload).unwrap();
        let mut offset: usize = 0;
        let count = CompactSizeUint::unmarshalling(&payload, &mut offset).unwrap();
        for _ in 0..count.decoded_value() as usize {

        }

    }
}


