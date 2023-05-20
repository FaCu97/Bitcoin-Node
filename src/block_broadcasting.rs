use std::{net::TcpStream, sync::{Arc, RwLock}, thread::{JoinHandle, self}, io::Read};
use crate::{block_header::BlockHeader, block::Block, messages::message_header::{HeaderMessage, command_name_to_bytes}, log_writer::{LogSender, write_in_log}, node};


const HEADERS: &str = "headers\0\0\0\0\0";






pub fn listen_for_incoming_blocks(log_sender: LogSender, mut nodes: Arc<RwLock<Vec<TcpStream>>>, headers: Vec<BlockHeader>, blocks: Vec<Block>)  {
    let mut nodes_handle: Vec<JoinHandle<()>> = vec![];
    println!("cantidad de nodos: {:?}", nodes.read().unwrap().len());
    for _ in 0..nodes.read().unwrap().len() {
        let node = nodes.write().unwrap().pop().unwrap();
        nodes_handle.push(listen_for_incoming_blocks_from_node(log_sender.clone(), node, headers.clone(), blocks.clone()))
    }
    for handle in nodes_handle {
        handle.join().unwrap();
    }
}


pub fn listen_for_incoming_blocks_from_node(log_sender: LogSender, mut node: TcpStream,  headers: Vec<BlockHeader>, blocks: Vec<Block>) -> JoinHandle<()> {
    let log_sender_clone = log_sender.clone();
    let t = thread::spawn(move || {
        loop {
            println!("Estoy esperando leer algo\n");
            let mut buffer_num = [0; 24];
            node.read_exact(&mut buffer_num).unwrap();
            let mut header = HeaderMessage::from_le_bytes(buffer_num).unwrap();
            let payload_size = header.payload_size as usize;
            let mut payload_buffer_num: Vec<u8> = vec![0; payload_size];
            node.read_exact(&mut payload_buffer_num).unwrap();
            println!("Lei algooo!!!!\n");
            let h = HEADERS.to_string();
            match header.command_name {
                h => write_in_log(log_sender_clone.info_log_sender.clone(), "ENCONTRE UN HEADERS!!!!"),
                _ => println!("Me llego otra cosa que no es un headers!"),
            }
        }

    });
    t
}