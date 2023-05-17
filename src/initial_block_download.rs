use std::error::Error;
use std::net::TcpStream;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, RwLock};
use std::{thread, vec, fmt};
use crate::{block::Block, block_header::BlockHeader};
use crate::messages::{block_message::BlockMessage , inventory::Inventory, get_data_message::GetDataMessage, getheaders_message::GetHeadersMessage, headers_message::HeadersMessage};
use crate::config::Config;
use chrono::{ TimeZone, Utc};
use std::io;


// todo: Cambiar la manera en que se pasa el config (?)
// todo: Pasar constantes a config 
// todo: Sacar unwraps 
// todo: Agregar validacion de headers
// todo: Si no se pudo descargar de un nodo, intentar descargar con otro (?)



#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DownloadError {
    CanNotJoin(String),
    CanNotRead(String),
    CanNotLock(String),
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DownloadError::CanNotJoin(msg) => write!(f, "CanNotJoin Error: {}", msg),
            DownloadError::CanNotRead(msg) => write!(f, "CanNotRead Error: {}", msg),
            DownloadError::CanNotLock(msg) => write!(f, "CanNotLock Error: {}", msg),
        }
    }
}

impl Error for DownloadError {}

// HASH DEL BLOQUE 2000000: [140, 59, 62, 211, 170, 119, 142, 174, 205, 203, 233, 29, 174, 87, 25, 124, 225, 186, 160, 215, 195, 62, 134, 208, 13, 1, 0, 0, 0, 0, 0, 0]
const GENESIS_BLOCK: [u8; 32] = [140, 59, 62, 211, 170, 119, 142, 174, 205, 203, 233, 29, 174, 87, 25, 124, 225, 186, 160, 215, 195, 62, 134, 208, 13, 1, 0, 0, 0, 0, 0, 0];

/* 
const GENESIS_BLOCK: [u8; 32] = 
[
    0x00, 0x00, 0x00, 0x00, 0x09, 0x33, 0xea, 0x01, 0xad, 0x0e, 0xe9, 0x84, 0x20, 0x97, 0x79,
    0xba, 0xae, 0xc3, 0xce, 0xd9, 0x0f, 0xa3, 0xf4, 0x08, 0x71, 0x95, 0x26, 0xf8, 0xd7, 0x7f,
    0x49, 0x43,
];
*/

const ALTURA_PRIMER_BLOQUE_A_DESCARGAR: usize = 428000;
const ALTURA_BLOQUES_A_DESCARGAR: usize = ALTURA_PRIMER_BLOQUE_A_DESCARGAR + 2000;
const FECHA_INICIO_PROYECTO: &str = "2023-04-10 00:06:14";
const FORMATO_FECHA_INICIO_PROYECTO: &str = "%Y-%m-%d %H:%M:%S";



pub fn search_first_header_block_to_download(headers: Vec<BlockHeader>, found: &mut bool) -> Result<Vec<BlockHeader>, Box<dyn Error>> {
    let fecha_hora = Utc.datetime_from_str(FECHA_INICIO_PROYECTO, FORMATO_FECHA_INICIO_PROYECTO)?;
    let timestamp = fecha_hora.timestamp() as u32;

    let mut first_headers_from_blocks_to_download = vec![];
    for header in headers {
        if !(*found) && header.time == timestamp {
            *found = true;
        } 
        if *found {
            first_headers_from_blocks_to_download.push(header);
        }
    }
    Ok(first_headers_from_blocks_to_download)
}



pub fn download_headers(config: Arc<RwLock<Config>>, nodes: Arc<RwLock<Vec<TcpStream>>>, headers: Arc<RwLock<Vec<BlockHeader>>>, tx: Sender<Vec<BlockHeader>>) -> Result<(), Box<dyn Error>>{
    let mut node = nodes.write().map_err(|err| DownloadError::CanNotLock(format!("{}", err)))?.pop().ok_or("Error no hay mas nodos para descargar los headers!\n")?;

    let config_guard = config.read().map_err(|err| DownloadError::CanNotLock(format!("{}", err)))?;
    


    let mut first_block_found = false;
    // write first getheaders message with genesis block
    GetHeadersMessage::build_getheaders_message(&config_guard, vec![GENESIS_BLOCK]).write_to(&mut node)?;
    // read first 2000 headers from headers message answered from node
    let mut headers_read = HeadersMessage::read_from(&mut node)?;
    // store headers in `global` vec `headers_guard`   
    headers.write().map_err(|err| DownloadError::CanNotLock(format!("{}", err)))?.extend_from_slice(&headers_read);
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

        if headers.read().map_err(|err| DownloadError::CanNotLock(format!("{}", err)))?.len() == ALTURA_PRIMER_BLOQUE_A_DESCARGAR {
            let first_block_headers_to_download = search_first_header_block_to_download(headers_read.clone(), &mut first_block_found)?;
            tx.send(first_block_headers_to_download)?;
        }
        if first_block_found && headers.read().map_err(|err| DownloadError::CanNotLock(format!("{}", err)))?.len() >= ALTURA_BLOQUES_A_DESCARGAR {
            tx.send(headers_read.clone())?;
            println!("ENVIO {:?} HEADERS\n", headers_read.len());
        }

        // store headers in `global` vec `headers_guard`
        headers.write().map_err(|err| DownloadError::CanNotLock(format!("{}", err)))?.extend_from_slice(&headers_read);
        println!("{:?}\n", headers.read().map_err(|err| DownloadError::CanNotLock(format!("{}", err)))?.len());    
    }
    nodes.write().map_err(|err| DownloadError::CanNotLock(format!("{}", err)))?.push(node);
    Ok(())
}




pub fn download_blocks(nodes: Arc<RwLock<Vec<TcpStream>>>, blocks: Arc<RwLock<Vec<Block>>>, rx: Receiver<Vec<BlockHeader>>) -> Result<(), Box<dyn Error>> {
    
    
    // recieves in the channel the vec of headers sent by the function downloading headers
    for recieved in rx {
        println!("RECIBO {:?} HEADERS\n", recieved.len());
        
        // divides the vec into 8 with the same lenght (or same lenght but the last with less)
        let chunk_size = (recieved.len() as f64 / 8_f64).ceil() as usize;
        let blocks_headers_chunks = Arc::new(RwLock::new(
            recieved
                .chunks(chunk_size)
                .map(|chunk| chunk.to_vec())
                .collect::<Vec<_>>(),
        ));
        let mut handle_join = vec![];
        for i in 0..8 {
            let nodes_pointer_clone = Arc::clone(&nodes);
            let block_headers_chunks_clone = Arc::clone(&blocks_headers_chunks);
            let blocks_pointer_clone = Arc::clone(&blocks);
            let mut node = nodes_pointer_clone.write().map_err(|err| DownloadError::CanNotLock(format!("{}", err)))?.pop().ok_or("No hay nodos para descargar los bloques!\n")?;
            handle_join.push(thread::spawn(move || {
                let block_headers = block_headers_chunks_clone.write().unwrap()[i].clone();
                println!("VOY A DESCARGAR {:?} BLOQUES DEL NODO {:?}\n", block_headers.len(), node);
                for chunk in block_headers.chunks(16) {
                    let mut inventory = vec![];
                    for block in chunk {
                        inventory.push(Inventory::new_block(block.hash()));
                    }
                    GetDataMessage::new(inventory).write_to(&mut node).unwrap();
                    for _ in 0..chunk.len() {
                        let bloque = BlockMessage::read_from(&mut node).unwrap();
                        println!("CANTIDAD DE BLOQUES DESCARGADOS: {:?}\n", blocks_pointer_clone.read().unwrap().len());
                        blocks_pointer_clone.write().unwrap().push(bloque);
                    }
                }
                nodes_pointer_clone.write().unwrap().push(node);
                }));
        }
        for h in handle_join {
            h.join().map_err(|err| DownloadError::CanNotJoin(format!("{:?}", err)))?;
        }
        
    }
    Ok(())

}







pub fn ibd(config: Config, nodes: Arc<RwLock<Vec<TcpStream>>>) -> Result<Vec<BlockHeader>, DownloadError> {
    
    let headers = vec![];
    let blocks: Vec<Block> = vec![];

    let (tx , rx ) = channel();


    let pointer_to_headers = Arc::new(RwLock::new(headers));
    let pointer_to_config = Arc::new(RwLock::new(config));
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


    let pointer_to_blocks = Arc::new(RwLock::new(blocks));
    let pointer_to_blocks_clone = Arc::clone(&pointer_to_blocks);
    let blocks_thread = thread::spawn(move || -> io::Result<()> {
        match download_blocks(nodes, pointer_to_blocks_clone, rx) {
            Err(e) => {
                Err(io::Error::new(io::ErrorKind::Other, e.to_string()))
                },
            Ok(_) => io::Result::Ok(()),
        }
    });
    headers_thread.join().map_err(|err| DownloadError::CanNotJoin(format!("{:?}", err)))?.map_err(|err| DownloadError::CanNotJoin(format!("{}", err)))?;  
    blocks_thread.join().map_err(|err| DownloadError::CanNotJoin(format!("{:?}", err)))?.map_err(|err| DownloadError::CanNotJoin(format!("{}", err)))?;


    let headers = &*pointer_to_headers.read().map_err(|err|DownloadError::CanNotLock(format!("{:?}", err)))?;
    let blocks = &*pointer_to_blocks.read().map_err(|err| DownloadError::CanNotLock(format!("{:?}", err)))?;
    println!("HEADERS DESCARGADOS: {:?}", headers.len());
    println!("BLOQUES A DESCARGAR: {:?}", blocks.len());
    println!("ULTIMO BLOQUE: {:?}", blocks.last().unwrap());

    Ok(headers.clone())
}





