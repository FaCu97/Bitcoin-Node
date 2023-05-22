/*
use crate::{
    block::Block,
    block_header::BlockHeader,
    log_writer::{write_in_log, LogSender},
    messages::{
        headers_message::HeadersMessage,
        message_header::{command_name_to_bytes, HeaderMessage},
    },
    node,
};
use std::{
    io::Read,
    net::TcpStream,
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
};

const HEADERS: &str = "headers\0\0\0\0\0";

pub fn listen_for_incoming_blocks(
    log_sender: LogSender,
    mut nodes: Arc<RwLock<Vec<TcpStream>>>,
    headers: Vec<BlockHeader>,
    blocks: Vec<Block>,
) {
    let mut nodes_handle: Vec<JoinHandle<()>> = vec![];
    println!("cantidad de nodos: {:?}", nodes.read().unwrap().len());
    for _ in 0..nodes.read().unwrap().len() {
        let node = nodes.write().unwrap().pop().unwrap();
        nodes_handle.push(listen_for_incoming_blocks_from_node(
            log_sender.clone(),
            node,
            headers.clone(),
            blocks.clone(),
        ))
    }
    for handle in nodes_handle {
        handle.join().unwrap();
    }
}

pub fn listen_for_incoming_blocks_from_node(
    log_sender: LogSender,
    mut node: TcpStream,
    headers: Vec<BlockHeader>,
    blocks: Vec<Block>,
) -> JoinHandle<()> {
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
*/
