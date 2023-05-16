use std::error::Error;
use std::net::TcpStream;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex, MutexGuard};
use std::{thread, vec};
use crate::{block::Block, block_header::BlockHeader};
use crate::messages::{block_message::BlockMessage ,inventory::Inventory, get_data_message::GetDataMessage, getheaders_message::GetHeadersMessage, headers_message::HeadersMessage};
use crate::config::Config;
use chrono::{ TimeZone, Utc};


// [120, 68, 126, 97, 111, 67, 237, 161, 95, 205, 185, 172, 158, 124, 192, 106, 14, 28, 5, 185, 250, 200, 168, 19, 38, 0, 0, 0, 0, 0, 0, 0];
// HASH DEL BLOQUE 2000000: [140, 59, 62, 211, 170, 119, 142, 174, 205, 203, 233, 29, 174, 87, 25, 124, 225, 186, 160, 215, 195, 62, 134, 208, 13, 1, 0, 0, 0, 0, 0, 0]
const GENESIS_BLOCK: [u8; 32] = [140, 59, 62, 211, 170, 119, 142, 174, 205, 203, 233, 29, 174, 87, 25, 124, 225, 186, 160, 215, 195, 62, 134, 208, 13, 1, 0, 0, 0, 0, 0, 0];

/* 
const GENESIS_BLOCK: [u8; 32] = 
[
    0x00, 0x00, 0x00, 0x00, 0x09, 0x33, 0xea, 0x01, 0xad, 0x0e, 0xe9, 0x84, 0x20, 0x97, 0x79,
    0xba, 0xae, 0xc3, 0xce, 0xd9, 0x0f, 0xa3, 0xf4, 0x08, 0x71, 0x95, 0x26, 0xf8, 0xd7, 0x7f,
    0x49, 0x43,
];*/

/* 
[
    0x00, 0x00, 0x00, 0x00, 0x09, 0x33, 0xea, 0x01, 0xad, 0x0e, 0xe9, 0x84, 0x20, 0x97, 0x79,
    0xba, 0xae, 0xc3, 0xce, 0xd9, 0x0f, 0xa3, 0xf4, 0x08, 0x71, 0x95, 0x26, 0xf8, 0xd7, 0x7f,
    0x49, 0x43,
];
*/



pub fn download_headers(config: Arc<Mutex<Config>>, mut node: TcpStream, headers: Arc<Mutex<Vec<BlockHeader>>>, tx: Sender<Vec<BlockHeader>>) -> Result<(), Box<dyn Error>>{
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
    headers_guard.extend(headers_read.clone());
    while headers_read.len() == 2000 {
        let last_header_hash = match headers_read.last() {
            Some(block_header) => Ok::<[u8; 32], Box<dyn Error>>(block_header.hash()) ,  
            None => Err("No se pudo obtener el ultimo elemento del vector de 2000 headers".into())
        }?;
        let getheaders_message = GetHeadersMessage::build_getheaders_message(&config_guard,vec![last_header_hash]);
        getheaders_message.write_to(&mut node)?;
        headers_read = HeadersMessage::read_from(&mut node)?;
        if headers_guard.len() == 428000 {
            let mut first_block_headers = vec![];
            for header in headers_read.clone() {
                if header.time == timestamp {
                    encontrado = true;
                    println!("LO ENCONTRE!\n");
                }
                if encontrado == true {
                    first_block_headers.push(header);
                }
            }
            tx.send(first_block_headers)?;
        }
        if encontrado == true && headers_guard.len() >= 430000{
            tx.send(headers_read.clone())?;
            println!("ENVIO {:?} HEADERS\n", headers_read.len());
        }
        headers_guard.extend(headers_read.clone());
        println!("{:?}\n", headers_guard.len());
    }
    Ok(())
}


pub fn download_blocks(blocks: Arc<Mutex<Vec<BlockHeader>>>, rx: Receiver<Vec<BlockHeader>>) -> Result<(), Box<dyn Error>> {
    for recieved in rx {
        println!("RECIBO {:?} HEADERS\n", recieved.len());
        let mut blocks_guard = match blocks.lock() {
            Ok(guard) => guard,
            Err(e) => {
                return Err(e.to_string().into())
            },
        };
        blocks_guard.extend(recieved.clone());
        let chunk_size = (recieved.len() as f64 / 8 as f64).ceil() as usize;
        let blocks_headers_chunks = Arc::new(Mutex::new(
            recieved
                .chunks(chunk_size)
                .map(|chunk| chunk.to_vec())
                .collect::<Vec<_>>(),
        ));


            /* 
            let mut thread_handles = vec![];
            let pointer_to_thread_handles = Arc::new(Mutex::new(thread_handles));
            let pointer_to_thread_handles_clone = Arc::clone(&pointer_to_thread_handles);
            for i in 1..9 {
                let n = nodes.remove(i);
                pointer_to_thread_handles_clone.lock().unwrap().push(thread::spawn(move || {
                    //let blocks_from_this_thread = bloques[i];
                
                }).join().unwrap());
                nodes.push(n);


            }
            */
            // divido los headers que me llegaron en 8 vectores de igual tamaño
            // creo 8 threads, cada uno conectado a un nodo distinto
            // 
            /* 
            for block in recieved {
                let block_hash = block.hash();
                let mut inventories = Vec::new();
                inventories.push(Inventory::new_block(block_hash));
                let data_message = GetDataMessage::new(inventories);
                data_message.write_to(&mut n).unwrap();
                let bloque = BlockMessage::read_from(&mut n).unwrap();
                pointer_to_blocks_clone.lock().unwrap().push(bloque);
            }
            */
    }
    Ok(())

}


pub fn ibm(config: Config, mut nodes: Vec<TcpStream>) -> Result<Vec<BlockHeader>, Box<dyn Error>> {
    

    let node = nodes.remove(0);
    let (tx, rx) = channel();
    let headers = vec![];
    let pointer_to_headers = Arc::new(Mutex::new(headers));
    let pointer_to_config = Arc::new(Mutex::new(config));
    let pointer_to_headers_clone = Arc::clone(&pointer_to_headers);
    let headers_thread = thread::spawn(move || {
        match download_headers(pointer_to_config, node, pointer_to_headers_clone, tx) {
            Err(e) => println!("{:?}", e),
            Ok(_) => (),
        };

    });





    
    let blocks: Vec<BlockHeader> = vec![];
    let pointer_to_blocks = Arc::new(Mutex::new(blocks));
    let pointer_to_blocks_clone = Arc::clone(&pointer_to_blocks);
    let blocks_thread = thread::spawn(move || {
        match download_blocks(pointer_to_blocks_clone, rx) {
            Err(e) => println!("{:?}", e),
            Ok(_) => (),
        }
        
        
    });



    headers_thread.join().unwrap();
    blocks_thread.join().unwrap();
    let headers = &*pointer_to_headers.lock().unwrap();
    let blocks = &*pointer_to_blocks.lock().unwrap();
    println!("HEADERS DESCARGADOS: {:?}", headers.len());
    println!("BLOQUES A DESCARGAR: {:?}", blocks.len());


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