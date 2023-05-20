use std::{net::TcpStream, sync::{Arc, RwLock}, thread::{JoinHandle, self}, io::Read};
use crate::{block_header::BlockHeader, block::Block, messages::{message_header::{HeaderMessage, command_name_to_bytes}, headers_message::HeadersMessage}, log_writer::{LogSender, write_in_log}, node};


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
            let header = HeadersMessage::read_from(log_sender_clone.clone(), &mut node).unwrap();
            // pedir bloques
        }

    });
    t
}