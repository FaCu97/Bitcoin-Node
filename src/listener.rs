use std::{net::TcpStream, sync::{RwLock, Arc}, io::Read};

use crate::{logwriter::log_writer::{LogSender, write_in_log}, messages::{message_header::HeaderMessage, headers_message::{is_terminated, HeadersMessage}}, blocks::block_header::BlockHeader};

pub fn listen_for_incoming_messages(
    log_sender: LogSender,
    stream: &mut TcpStream,
    finish: Option<Arc<RwLock<bool>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer_num = [0; 24];
    stream.read_exact(&mut buffer_num)?;
    let mut header = HeaderMessage::from_le_bytes(buffer_num)?;
    while !is_terminated(finish.clone()) {
        let payload_size = header.payload_size as usize;
        let mut payload_buffer_num: Vec<u8> = vec![0; payload_size];
        stream.read_exact(&mut payload_buffer_num)?;
        match header.command_name.as_str() {
            "headers" => handle_headers_message(payload_buffer_num),
            "inv" => {
                println!("recibo inv!\n");
            },
            "ping" => handle_ping_message(payload_buffer_num),
            _ => {
                write_in_log(
                    log_sender.messege_log_sender.clone(),
                    format!(
                        "IGNORADO -- Recibo: {} -- Nodo: {:?}",
                        header.command_name,
                        stream.peer_addr()?
                    )
                    .as_str(),
                );
            }
        }
        buffer_num = [0; 24];
        stream.read_exact(&mut buffer_num)?;
        header = HeaderMessage::from_le_bytes(buffer_num)?;
    }
    Ok(())
}



pub fn handle_headers_message(payload: Vec<u8>) -> Result<Vec<BlockHeader>, &'static str> {
    HeadersMessage::unmarshalling(&payload)
}


pub fn handle_ping_message(payload: Vec<u8>) {
    write_pong_message(node, )
}
/* 
pub fn handle_inv_message(payload: Vec<u8>) {
    
}*/