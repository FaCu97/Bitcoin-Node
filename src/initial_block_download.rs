use std::error::Error;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::{block::Block, block_header::BlockHeader};
use crate::messages::{block_message::BlockMessage ,inventory::Inventory, get_data_message::GetDataMessage, getheaders_message::GetHeadersMessage, headers_message::HeadersMessage};
use crate::config::Config;
use chrono::{ TimeZone, Utc};


// [120, 68, 126, 97, 111, 67, 237, 161, 95, 205, 185, 172, 158, 124, 192, 106, 14, 28, 5, 185, 250, 200, 168, 19, 38, 0, 0, 0, 0, 0, 0, 0];
// HASH DEL BLOQUE 2000000: [140, 59, 62, 211, 170, 119, 142, 174, 205, 203, 233, 29, 174, 87, 25, 124, 225, 186, 160, 215, 195, 62, 134, 208, 13, 1, 0, 0, 0, 0, 0, 0]
const GENESIS_BLOCK: [u8; 32] = [140, 59, 62, 211, 170, 119, 142, 174, 205, 203, 233, 29, 174, 87, 25, 124, 225, 186, 160, 215, 195, 62, 134, 208, 13, 1, 0, 0, 0, 0, 0, 0];

/* 
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
pub fn ibm(config: Config, mut nodes: Vec<TcpStream>) -> Result<Vec<BlockHeader>, Box<dyn Error>> {
    // *********************************************
    // *******   timestamp primer bloque   *********
    // *********************************************
    let fecha_hora_str = "2023-04-10 00:06:14";
    let formato = "%Y-%m-%d %H:%M:%S";
    let fecha_hora = Utc.datetime_from_str(fecha_hora_str, formato)?;
    let timestamp = fecha_hora.timestamp() as u32;

    let mut node = nodes.remove(0);
   
    let mut headers = vec![];
    let mut blocks = vec![];
    let blocks_lock: Arc<Mutex<Vec<BlockHeader>>> = Arc::new(Mutex::new(blocks));
    let blocks_lock_cloned = Arc::clone(&blocks_lock);

    let headers_lock: Arc<Mutex<Vec<BlockHeader>>> = Arc::new(Mutex::new(headers));
    let headers_lock_cloned = Arc::clone(&headers_lock);
    
    let headers_thread = thread::spawn(move || {
        let configuracion = config.clone();

        let mut encontrado = false;
        GetHeadersMessage::build_getheaders_message(config.clone(), vec![GENESIS_BLOCK]).write_to(&mut node).unwrap();
        let mut headers_read = HeadersMessage::read_from(&mut node).unwrap();
        headers_lock_cloned.lock().unwrap().extend(headers_read.clone());
        while headers_read.len() == 2000 {
            let last_header_hash = headers_read.last().unwrap().hash();
            let getheaders_message = GetHeadersMessage::build_getheaders_message(configuracion.clone(),vec![last_header_hash]);
            getheaders_message.write_to(&mut node).unwrap();
            headers_read = HeadersMessage::read_from(&mut node).unwrap();
            if headers_lock_cloned.lock().unwrap().len() == 428000 {
                for header in headers_read.clone() {
                    if header.time == timestamp {
                        encontrado = true;
                        println!("LO ENCONTRE!\n");
                    }
                    if encontrado == true {
                        blocks_lock_cloned.lock().unwrap().push(header);
                    }
                }
            }
            if encontrado == true && headers_lock_cloned.lock().unwrap().len() >= 430000{
                blocks_lock_cloned.lock().unwrap().extend(headers_read.clone());
            }
            headers_lock_cloned.lock().unwrap().extend(headers_read.clone());
            println!("{:?}\n", headers_lock_cloned.lock().unwrap().len());
    
        }
      
    });
    headers_thread.join().unwrap();
    let headers = Arc::try_unwrap(headers_lock).unwrap().into_inner().unwrap();
    let blocks = Arc::try_unwrap(blocks_lock).unwrap().into_inner().unwrap();
    println!("HEADERS DESCARGADOS: {:?}", headers.len());
    println!("HEADERS DE BLOQUES A DESCARGAR: {:?}", blocks.len());

    // DESCARGA DE BLOQUES
    let mut n = nodes.remove(1);
    let mut bloques_descargados: Vec<Block> = vec![];
    let mut c = 0;
    for block in blocks.clone() {
        c += 1;
        if c < 4000 {
            continue;
        }
        let block_hash = block.hash();
        let mut inventories = Vec::new();
        inventories.push(Inventory::new_block(block_hash));
        let data_message = GetDataMessage::new(inventories);
        data_message.write_to(&mut n)?;
        let bloque = BlockMessage::read_from(&mut n)?;
        println!("BLOQUE DESCARGADO: {:?}\n", bloque);
        bloques_descargados.push(bloque);
        println!("{:?}\n", bloques_descargados.len());
    }
    println!("CANTIDAD DE BLOQUES DESCARGADOS: {:?} \n", bloques_descargados.len());
    println!("ULTIMO BLOQUE DESCARGADO: {:?} \n", bloques_descargados.last().unwrap());




    Ok(headers)
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