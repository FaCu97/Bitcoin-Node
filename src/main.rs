use bitcoin::config::Config;
use bitcoin::messages::get_data_message::GetDataMessage;
use bitcoin::messages::message_header::write_verack_message;
use bitcoin::messages::{
    message_header::HeaderMessage,
    version_message::{get_version_message, VersionMessage},
};
use bitcoin::network::get_active_nodes_from_dns_seed;
use std::error::Error;
use std::io::Read;
use std::net::{SocketAddr, TcpStream, Ipv4Addr, SocketAddrV4};
use std::process::exit;
use std::result::Result;
use std::str::Utf8Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{env, thread};

fn main() {
    let args: Vec<String> = env::args().collect();
    let config: Config = match Config::from(&args) {
        Err(e) => {
            println!("Application error: {e}");
            exit(1)
        }
        Ok(config) => config,
    };
    let active_nodes = match get_active_nodes_from_dns_seed(&config) {
        Err(e) => {
            println!("ERROR: {}", e);
            exit(-1)
        }
        Ok(active_nodes) => active_nodes,
    };

    let sockets: Vec<TcpStream> = handshake(config.clone(), &active_nodes);

    println!("Sockets: {:?}", sockets);
    println!("CANTIDAD SOCKETS: {:?}", sockets.len());
    println!("{:?}", config.user_agent);
    // Acá iría la descarga de los headers
}

fn handshake(config: Config, active_nodes: &[Ipv4Addr]) -> Vec<TcpStream> {
    let lista_nodos = Arc::new(active_nodes);
    let chunk_size = (lista_nodos.len() as f64 / 1 as f64).ceil() as usize;
    let active_nodes_chunks = Arc::new(Mutex::new(
        lista_nodos
            .chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect::<Vec<_>>(),
    ));
    let sockets = vec![];
    let sockets_lock = Arc::new(Mutex::new(sockets));
    let mut thread_handles = vec![];

    for i in 0..1 {
        let chunk = active_nodes_chunks.lock().unwrap()[i].clone();
        let configuracion = config.clone();
        let sockets: Arc<Mutex<Vec<TcpStream>>> = Arc::clone(&sockets_lock);
        thread_handles.push(thread::spawn(move || {
            conectar_a_nodo(configuracion, sockets, &chunk);
        }));
    }
    println!("{:?}", sockets_lock.lock().unwrap().len());
    for handle in thread_handles {
        handle.join().unwrap();
        //  sockets.extend(result);
    }
    Arc::try_unwrap(sockets_lock).unwrap().into_inner().unwrap()
}

// los threads no pueden manejar un dyn Error
// En el libro devuelve thread::Result<std::io::Result<()>>
fn conectar_a_nodo(configuracion: Config, sockets: Arc<Mutex<Vec<TcpStream>>>, nodos: &[Ipv4Addr]) {
    for nodo in nodos {
        match connect_to_node(&configuracion, nodo) {
            Ok(stream) => {
                println!("Conectado correctamente a: {:?} \n", nodo);
                sockets.lock().unwrap().push(stream);
            }
            Err(err) => {
                println!(
                    "Error {:?}. No se pudo conectar a: {:?}, voy a intenar conectarme a otro \n",
                    err, nodo
                );
            }
        };
        //    println!("CANTIDAD SOCKETS: {:?}", sockets.lock().unwrap().len());
    }
}

fn connect_to_node(config: &Config, node_ip: &Ipv4Addr) -> Result<TcpStream, Box<dyn Error>> {
    //let socket_addr: SocketAddr = node_ip.parse()?;
    let port:u16 = 18333;
    let socket_addr = SocketAddr::new(node_ip.clone().into(), port);
    let mut stream: TcpStream = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(5))?;
     
    let local_ip_addr = stream.local_addr()?;
    let version_message = get_version_message(config, socket_addr, local_ip_addr)?;
    version_message.write_to(&mut stream)?;
    let version_response = VersionMessage::read_from(&mut stream)?;
//    println!(
//        "RECIBO MENSAJE VERSION DEL NODO {:?}: {:?}\n",
//        node_ip, version_response
//    );
    
    let verack_response = write_verack_message(&mut stream)?;
//    println!(
//        "RECIBO MENSAJE VERACK DEL NODO {:?}: {:?}\n",
//        node_ip, verack_response
//    );
    //f2b614c393c2d428b79021c7a0cf1d0e418a54224856
    let vec:Vec<[u8;32]> = vec![
        [
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x13,
            0x16, 0x2b, 0xf2, 0xb6,
            0x14, 0xc3, 0x93, 0xc2,
            0xd4, 0x28, 0xb7, 0x90,
            0x21, 0xc7, 0xa0, 0xcf,
            0x1d, 0x0e, 0x41, 0x8a,
            0x54, 0x22, 0x48, 0x56,
    ]
];
    let get_data_message = GetDataMessage::new(vec);
    get_data_message.write_to(&mut stream)?;
    println!("getdata: {:?}", get_data_message);
    for _ in 0..6 {
        let mut buffer_num = [0; 24];
        stream.read_exact(&mut buffer_num)?;
        let header = HeaderMessage::from_le_bytes(buffer_num).map_err(|err: Utf8Error| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
        })?;
        let payload_large = header.payload_size;
        let mut buffer_num = vec![0; payload_large as usize];
        stream.read_exact(&mut buffer_num)?;
        println!(
            "RECIBO MENSAJE HEADER DEL NODO {:?}: {:?}\n",
            node_ip, header
        );
    }

    
/* 
    let verack_message = get_verack_message(config);
    verack_message.write_to(&mut stream)?;
    let verack_response = NonePayloadMessage::read_from(&mut stream)?;
    println!(
        "RECIBO MENSAJE VERACK DEL NODO {:?}: {:?}\n",
        node_ip, verack_response
    );
*/ 
    
    Ok(stream)
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_archivo_configuracion() {}
}
