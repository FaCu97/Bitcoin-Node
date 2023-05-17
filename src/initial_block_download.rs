use std::error::Error;
use std::net::TcpStream;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::{thread, vec};
use crate::{block::Block, block_header::BlockHeader};
use crate::messages::{block_message::BlockMessage ,inventory::Inventory, get_data_message::GetDataMessage, getheaders_message::GetHeadersMessage, headers_message::HeadersMessage};
use crate::config::Config;
use chrono::{ TimeZone, Utc};
use std::io;


// HASH DEL BLOQUE 2000000: [140, 59, 62, 211, 170, 119, 142, 174, 205, 203, 233, 29, 174, 87, 25, 124, 225, 186, 160, 215, 195, 62, 134, 208, 13, 1, 0, 0, 0, 0, 0, 0]
const GENESIS_BLOCK: [u8; 32] = [140, 59, 62, 211, 170, 119, 142, 174, 205, 203, 233, 29, 174, 87, 25, 124, 225, 186, 160, 215, 195, 62, 134, 208, 13, 1, 0, 0, 0, 0, 0, 0];

/* 
const GENESIS_BLOCK: [u8; 32] = 
[
    0x00, 0x00, 0x00, 0x00, 0x09, 0x33, 0xea, 0x01, 0xad, 0x0e, 0xe9, 0x84, 0x20, 0x97, 0x79,
    0xba, 0xae, 0xc3, 0xce, 0xd9, 0x0f, 0xa3, 0xf4, 0x08, 0x71, 0x95, 0x26, 0xf8, 0xd7, 0x7f,
    0x49, 0x43,
];*/

const ALTURA_PRIMER_BLOQUE_A_DESCARGAR: usize = 428000;
const ALTURA_BLOQUES_A_DESCARGAR: usize = ALTURA_PRIMER_BLOQUE_A_DESCARGAR + 2000;
const FECHA_INICIO_PROYECTO: &str = "2023-04-10 00:06:14";
const FORMATO_FECHA_INICIO_PROYECTO: &str = "%Y-%m-%d %H:%M:%S";



pub fn search_first_header_block_to_download(headers: Vec<BlockHeader>, found: &mut bool) -> Result<Vec<BlockHeader>, Box<dyn Error>> {
    // *********************************************
    // *******   timestamp primer bloque   *********
    // *********************************************
    let fecha_hora = Utc.datetime_from_str(FECHA_INICIO_PROYECTO, FORMATO_FECHA_INICIO_PROYECTO)?;
    let timestamp = fecha_hora.timestamp() as u32;

    let mut first_headers_from_blocks_to_download = vec![];
    for header in headers {
        if !(*found) && header.time == timestamp {
            *found = true;
            println!("ENCONTRADO!!!! \n");
        } 
        if *found {
            first_headers_from_blocks_to_download.push(header);
        }
    }
    Ok(first_headers_from_blocks_to_download)
}



pub fn download_headers(config: Arc<Mutex<Config>>, nodes: Arc<Mutex<Vec<TcpStream>>>, headers: Arc<Mutex<Vec<BlockHeader>>>, tx: Sender<Vec<BlockHeader>>) -> Result<(), Box<dyn Error>>{
    
    let mut node = nodes.lock().unwrap().pop().unwrap();

    let config_guard = match config.lock() {
        Ok(guard) => guard,
        Err(e) => {
            return Err(e.to_string().into())
        } 
    };
    
    let mut headers_guard = match headers.lock() {
        Ok(guard) => guard,
        Err(e) => {
            return Err(e.to_string().into())
        } 
    };


    let mut first_block_found = false;
    // write first getheaders message with genesis block
    GetHeadersMessage::build_getheaders_message(&config_guard, vec![GENESIS_BLOCK]).write_to(&mut node)?;
    // read first 2000 headers from headers message answered from node
    let mut headers_read = HeadersMessage::read_from(&mut node)?;
    // store headers in `global` vec `headers_guard`
    headers_guard.extend_from_slice(&headers_read);
    while  headers_read.len() == 2000 {
        // get the last header hash from the latest headers you have
        let last_header_hash = headers_read
            .last()
            .ok_or("No se pudo obtener el último elemento del vector de 2000 headers")?
            .hash();
        // write getheaders message with last header you have, asking for next 2000 (or less if they are the last ones)
        GetHeadersMessage::build_getheaders_message(&config_guard,vec![last_header_hash]).write_to(&mut node)?;
        // read next 2000 headers (or less if they are the last ones)
        headers_read = HeadersMessage::read_from(&mut node)?;

        
        if headers_guard.len() == ALTURA_PRIMER_BLOQUE_A_DESCARGAR {
            let first_block_headers_to_download = search_first_header_block_to_download(headers_read.clone(), &mut first_block_found)?;
            tx.send(first_block_headers_to_download)?;
        } 
        if first_block_found && headers_guard.len() >= ALTURA_BLOQUES_A_DESCARGAR {
            println!("ENVIO {:?} HEADERS A DESCARGAR SUS BLOQUES\n", headers_read.len());           
            tx.send(headers_read.clone())?;
        }

        // store headers in `global` vec `headers_guard`
        headers_guard.extend_from_slice(&headers_read);
        println!("HEADERS DESCARGADOS: {:?}\n", headers_guard.len());    
    }
    nodes.lock().unwrap().push(node);
    Ok(())
}








pub fn download_blocks(nodes: Arc<Mutex<Vec<TcpStream>>>, blocks: Arc<Mutex<Vec<Block>>>, rx: Receiver<Vec<BlockHeader>>) -> Result<(), Box<dyn Error>> {
    
    

    for recieved in rx {
        println!("RECIBO {:?} HEADERS\n", recieved.len());
        
         
        let chunk_size = (recieved.len() as f64 / 8_f64).ceil() as usize;
        let blocks_headers_chunks = Arc::new(Mutex::new(
            recieved
                .chunks(chunk_size)
                .map(|chunk| chunk.to_vec())
                .collect::<Vec<_>>(),
        ));
        let mut handle_join = vec![];
        for i in 0..8 {
            let pointer_cloned = nodes.clone();
            let mut n = pointer_cloned.lock().unwrap().pop().unwrap();
            let block_headers_chunk_clone = Arc::clone(&blocks_headers_chunks);
            let block_clone = Arc::clone(&blocks);
            handle_join.push(thread::spawn(move || {
                let chunk = block_headers_chunk_clone.lock().unwrap()[i].clone();
                println!("VOY A DESCARGAR {:?} BLOQUES DEL NODO {:?}\n", chunk.len(), n);
                for block in chunk {
                    let block_hash = block.hash();
                    let inventories = vec![Inventory::new_block(block_hash)];
                    let data_message = GetDataMessage::new(inventories);
                    data_message.write_to(&mut n).unwrap();
                    let bloque = BlockMessage::read_from(&mut n).unwrap();
                    //println!("CANTIDAD DE BLOQUES DESCARGADOS: {:?}\n", block_clone.lock().unwrap().len());
                    block_clone.lock().unwrap().push(bloque);
                }
                pointer_cloned.lock().unwrap().push(n);
                }));

            
        }
        for handle in handle_join {
            handle.join().unwrap();
        }
        
    }
    println!("SE CIERRA EL CHANNEL !!!!\n");
    Ok(())

}




pub fn ibd(config: Config, nodes: Arc<Mutex<Vec<TcpStream>>>) -> Result<Vec<BlockHeader>, Box<dyn Error>> {
    
    let headers = vec![];
    let blocks: Vec<Block> = vec![];

    let (tx , rx ) = channel();


    let pointer_to_headers = Arc::new(Mutex::new(headers));
    let pointer_to_config = Arc::new(Mutex::new(config));
    let pointer_to_headers_clone = Arc::clone(&pointer_to_headers);
    let pointer_to_nodes_clone = Arc::clone(&nodes);
    let headers_thread = thread::spawn(move || -> io::Result<()> {
        match download_headers(pointer_to_config, pointer_to_nodes_clone, pointer_to_headers_clone, tx) {
            Err(e) => {
                Err(io::Error::new(io::ErrorKind::Other, e.to_string()))
                },
            Ok(_) => io::Result::Ok(()),
        }
    });


    let pointer_to_blocks = Arc::new(Mutex::new(blocks));
    let pointer_to_blocks_clone = Arc::clone(&pointer_to_blocks);
    let blocks_thread = thread::spawn(move || -> io::Result<()> {
        match download_blocks(nodes, pointer_to_blocks_clone, rx) {
            Err(e) => {
                Err(io::Error::new(io::ErrorKind::Other, e.to_string()))
                },
            Ok(_) => io::Result::Ok(()),
        }
    });

    headers_thread.join().unwrap()?;    
    blocks_thread.join().unwrap()?;
    let headers = &*pointer_to_headers.lock().unwrap();
    let blocks = &*pointer_to_blocks.lock().unwrap();
    println!("HEADERS DESCARGADOS: {:?}", headers.len());
    println!("BLOQUES A DESCARGAR: {:?}", blocks.len());
    println!("ULTIMO BLOQUE: {:?}", blocks.last().unwrap());

    Ok(headers.clone())
}














/* 

pub fn download_headers(config: &Config, nodes: &mut Vec<TcpStream>) -> Result<Vec<BlockHeader>, Box<dyn Error>> {
    // *********************************************
    // *******   timestamp primer bloque   *********
    // *********************************************
    let fecha_hora_str = "2023-04-10 00:06:14";
    let formato = "%Y-%m-%d %H:%M:%S";
    let fecha_hora = Utc.datetime_from_str(fecha_hora_str, formato)?;
    let timestamp = fecha_hora.timestamp() as u32;

    
    
    let node = &mut nodes[0]; // agarro el primer nodo

    let mut headers_list: Vec<BlockHeader> = vec![]; // lista de headers
    let mut bloques: Vec<BlockHeader> = vec![]; // lista de bloques
    let mut encontrado = false;
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
        // si ya voy descargados 2428000 headers, busco en estos 2000 que me llegan el
        // header del primer bloque a descargar
        if headers_list.len() == 2428000 {
            for header in headers.clone() {
                if header.time == timestamp {
                    encontrado = true;
                    println!("LO ENCONTRE!\n");
                }
                if encontrado == true {
                    bloques.push(header);
                }
            }
        }
        if encontrado == true && headers_list.len() >= 2430000{
            bloques.extend(headers.clone());
        }
        headers_list.extend(headers.clone());
        println!("{:?}\n", headers_list.len());
    }
    println!("HEADERS DESCARGADOS: {:?}", headers_list.len());
    println!("HEADERS DE BLOQUES A DESCARGAR: {:?}", bloques.len());
    
    let mut bloques_descargados: Vec<Block> = vec![];
    let mut c = 0;
    for block in bloques.clone() {
        c += 1;
        if c < 4000 {
            continue;
        }
        let block_hash = block.hash();
        let mut inventories = Vec::new();
        inventories.push(Inventory::new_block(block_hash));
        let data_message = GetDataMessage::new(inventories);
        data_message.write_to(node)?;
        bloques_descargados.push(BlockMessage::read_from(node)?);
        println!("{:?}\n", c);
        println!("{:?}\n", bloques_descargados.len());
    }
    println!("CANTIDAD DE BLOQUES DESCARGADOS: {:?} \n", bloques_descargados.len());
    println!("ULTIMO BLOQUE DESCARGADO: {:?} \n", bloques_descargados.last().unwrap());
    
    Ok(headers_list)
}
*/


/*
Me conecto al primer nodo
Mando el primer mensaje getheaders con bloque genesis hardcodeado
Mientras la cantidad de headers que me llegan sean 2000, pido mas con el ultimo header que tengo
Valido parcialmente estos 2000 headers que me llegaron
Si son validos los agrego a la lista de headers de toda la blockchain
Si alguno es invalido o algun mensaje se lee mal/falla el nodo conectado, se conecta al siguiente
Si ya me conecte a todos y todavia no pude descargarme todos los headers se lanza error
Cuando llego al primer header del primer bloque que tengo que descargar, los valido y si ya estan validados
los envio mendiante un channel a otro thread para que se descarguen esos bloques.
me conecta a 8 nodos y con los headers que van llegando para descargar bloques los divido en 8 threads
cada thread le pide a un nodo los distitos bloques y los va agregando a una lista









 */









/*

 pub fn _download_headers(config: Arc<Mutex<Config>>, mut node: TcpStream, headers: Arc<Mutex<Vec<BlockHeader>>>, tx: Sender<Vec<BlockHeader>>) -> Result<(), Box<dyn Error>>{
    // *********************************************
    // *******   timestamp primer bloque   *********
    // *********************************************
    let fecha_hora_str = "2023-04-10 00:06:14";
    let formato = "%Y-%m-%d %H:%M:%S";
    let fecha_hora = Utc.datetime_from_str(fecha_hora_str, formato)?;
    let timestamp = fecha_hora.timestamp() as u32;
    
    
    let config_guard = match config.lock() {
        Ok(guard) => guard,
        Err(e) => {
            return Err(e.to_string().into())
        } 
    };
    let mut headers_guard = match headers.lock() {
        Ok(guard) => guard,
        Err(e) => {
            return Err(e.to_string().into())
        } 
    };
    let mut encontrado = false;
    GetHeadersMessage::build_getheaders_message(&config_guard, vec![GENESIS_BLOCK]).write_to(&mut node)?;
    let mut headers_read = HeadersMessage::read_from(&mut node)?;
    headers_guard.extend_from_slice(&headers_read);
    let headers_read_lenght : &usize = &headers_read.len();
    while  *headers_read_lenght== 2000 {
        let last_header_hash = match headers_read.last() {
            Some(block_header) => Ok::<[u8; 32], Box<dyn Error>>(block_header.hash()) ,  
            None => Err("No se pudo obtener el ultimo elemento del vector de 2000 headers".into())
        }?;
        let getheaders_message = GetHeadersMessage::build_getheaders_message(&config_guard,vec![last_header_hash]);
        getheaders_message.write_to(&mut node)?;
        headers_read = HeadersMessage::read_from(&mut node)?;
        if headers_guard.len() == 428000 {
            let mut first_block_headers: Vec<BlockHeader> = Vec::new();
            for header in &headers_read{
                if header.time == timestamp {
                    encontrado = true;
                    println!("LO ENCONTRE!\n");
                }
                if encontrado == true {
                    first_block_headers.push(*header);
                }
            tx.send(first_block_headers)?;
            }
        }
        if encontrado == true && headers_guard.len() >= 430000 {
            tx.send(headers_read.clone())?;
            println!("ENVIO {:?} HEADERS\n",headers_read_lenght);
        }
        headers_guard.extend_from_slice(&headers_read);
        println!("{:?}\n", headers_guard.len());    
    }
    Ok(())
}*/


