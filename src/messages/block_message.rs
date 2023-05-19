use std::{error::Error, io::Read, net::TcpStream};

use crate::block::Block;

use super::message_header::HeaderMessage;

/// Representa el mensaje block que se recibe en respuesta al mensaje getdata
#[derive(Debug)]
pub struct BlockMessage;

impl BlockMessage {
    /// Recibe en bytes el mensaje "block".
    /// Devuelve un bloque
    fn unmarshalling(block_message_payload_bytes: &Vec<u8>) -> Result<Block, Box<dyn Error>> {
        let mut offset = 0;
        let block = Block::unmarshalling(block_message_payload_bytes, &mut offset)?;
        Ok(block)
    }
    /// Dado un stream que implementa el trait Read (desde donde se puede leer) lee el mensaje block y devuelve
    /// el bloque correspondiente si se pudo leer correctamente o un Error en caso contrario.
    pub fn read_from(stream: &mut TcpStream) -> Result<Block, Box<dyn std::error::Error>> {
        let header = HeaderMessage::read_from(stream, "block".to_string())?;
        //println!("Header recibido: {:?}\n", header);
        let payload_size = header.payload_size as usize;
        let mut buffer_num = vec![0; payload_size];
        stream.read_exact(&mut buffer_num)?;
        let mut block_message_payload_bytes: Vec<u8> = vec![];
        block_message_payload_bytes.extend_from_slice(&buffer_num);
        let block = Self::unmarshalling(&block_message_payload_bytes)?;
        Ok(block)
    }
}
