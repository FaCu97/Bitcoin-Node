use core::time;
use std::error::Error;
use std::net::TcpStream;
use crate::{block::Block, block_header::BlockHeader};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::messages::{getheaders_message::GetHeadersMessage, headers_message::HeadersMessage};
use crate::config::Config;
use chrono::{DateTime, TimeZone, Utc};


pub fn initial_block_download(config: &Config, nodes: &mut Vec<TcpStream>, headers_list: &mut Vec<BlockHeader>, blocks: &mut Vec<Block>) -> Result<(), Box<dyn Error>> {
    
    Ok(())
}


pub fn download_headers(config: &Config, nodes: &mut Vec<TcpStream>) -> Result<Vec<BlockHeader>, Box<dyn Error>> {
    let fecha_hora_str = "2023-04-10 00:06:14";
    let formato = "%Y-%m-%d %H:%M:%S";
    let fecha_hora = Utc.datetime_from_str(fecha_hora_str, formato).unwrap();
    let timestamp = fecha_hora.timestamp() as u32;

    
    
    let node = &mut nodes[0];
    let mut headers_list: Vec<BlockHeader> = vec![];
    let genesis_locator_hash = vec![[
        0x00, 0x00, 0x00, 0x00, 0x09, 0x33, 0xea, 0x01, 0xad, 0x0e, 0xe9, 0x84, 0x20, 0x97, 0x79,
        0xba, 0xae, 0xc3, 0xce, 0xd9, 0x0f, 0xa3, 0xf4, 0x08, 0x71, 0x95, 0x26, 0xf8, 0xd7, 0x7f,
        0x49, 0x43,
    ]];
    
    GetHeadersMessage::build_getheaders_message(config, genesis_locator_hash).write_to(node)?;
    let mut headers = HeadersMessage::read_from(node)?;
    headers_list.extend(headers.clone());
    while headers.len() == 2000 {
        let last_header_hash = headers.last().unwrap().hash();
        let getheaders_message = GetHeadersMessage::build_getheaders_message(config,vec![last_header_hash]);
        getheaders_message.write_to(node)?;
        headers = HeadersMessage::read_from(node)?;
        if headers.len() < 2000 {
            println!(
                "HEADERS DEVUELVE MENOS DE 2000 HEADERS, ME DEVUELVE: {:?}\n",
                headers.len()
            );
        }
        headers_list.extend(headers.clone());
        println!("{:?}\n", headers_list.len());
    }
    //println!("ULTIMO HEADERS DEVUELTO POR NODO: {:?}\n", headers);
    println!(
        "CANTIDAD DE HEADERS DESCARGADOS: {:?}\n",
        headers_list.len()
    );
    let mut bloques: Vec<BlockHeader> = vec![];
    let mut encontrado = false;
    for block in headers_list.clone() {
        if block.time == timestamp {
            encontrado = true;
            println!("ENCONTRADOOOO!!!!!\n")
        }
        if encontrado == true {
            bloques.push(block);
        }
    }
    println!("BLOQUES A DESCARGAR {:?}", bloques.len());
    let mut bloques_descargados: Vec<Block> = vec![];
    
    Ok(headers_list)
}



