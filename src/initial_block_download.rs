use std::error::Error;
use std::net::TcpStream;
use crate::{block::Block, block_header::BlockHeader};
use std::sync::mpsc;
use std::thread;

pub fn initial_block_download(nodes: &mut Vec<TcpStream>,headers: &mut Vec<BlockHeader>, blocks: &mut Vec<Block>) -> Result<(), Box<dyn Error>> {
    let mut number_of_node = 0;
    let node = &mut nodes[0];
    
    
    Ok(())
}