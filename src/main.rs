use bitcoin::config::Config;
use bitcoin::messages::{
    message_header::HeaderMessage,
    none_payload_message::NonePayloadMessage,
    version_message::{get_version_message, VersionMessage},
};
use bitcoin::network::get_active_nodes_from_dns_seed;
use std::collections::VecDeque;
use std::error::Error;
use std::net::{SocketAddr, TcpStream};
use std::ops::DerefMut;
use std::process::exit;
use std::result::Result;
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

    /*
      let vec: Vec<i32> = vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15];

      let chunk_size = (vec.len() as f64 / 4 as f64).ceil() as usize;
      let chunks = vec.chunks(chunk_size).collect::<Vec<_>>();

     // let largo = vec.len()/8 ;
      println!("chunk_size: {:?}",chunk_size);
    //  let output = vec.split_off(3);
      //let output: Vec<&[i32]> = vec.chunks(largo).collect();

      println!("Chunks: {:?}", chunks);
      println!("{:?}", active_nodes);
     */
    let mut sockets: Vec<TcpStream> = handshake(config.clone(), active_nodes);

    println!("Sockets: {:?}", sockets);

    // Acá iría la descarga de los headers
}

fn handshake(config: Config, active_nodes: VecDeque<String>) -> Vec<TcpStream> {
    let mut sockets = Vec::new();
    let active_nodes_lock = Arc::new(Mutex::new(active_nodes.clone()));
    //let configuracion_lock = Arc::new(config.clone());
    // let active_nodes_lock_ref = active_nodes_lock.clone();
    let sockets_lock = Arc::new(Mutex::new(sockets));
    let mut thread_handles = vec![];

    let NTHREADS = 8; // pasar a Config
    for _ in 0..NTHREADS {
        let configuracion = config.clone();
        let active_nodes = Arc::clone(&active_nodes_lock);
        //let configuracion = Arc::clone(&configuracion_lock);
        let sockets: Arc<Mutex<Vec<TcpStream>>> = Arc::clone(&sockets_lock);
        thread_handles.push(thread::spawn(move || {
            conectar_a_nodo(configuracion, &active_nodes, sockets)
        }));
    }
    println!("{:?}", sockets_lock.lock().unwrap().len());
    for handle in thread_handles {
        handle.join().unwrap();
    }

    let sockets = Arc::try_unwrap(sockets_lock).unwrap().into_inner().unwrap();
    sockets

    //let sockets_guard = sockets_lock.lock().unwrap().deref_mut();
    //sockets_guard
    //sockets_guard.into_inner()
    //let sockets_vec = std::mem::replace(sockets_guard, Vec::new());
    //sockets_vec
}

// los threads no pueden manejar un dyn Error
// En el libro devuelve thread::Result<std::io::Result<()>>
fn conectar_a_nodo(
    configuracion: Config,
    active_nodes_ips: &Arc<Mutex<VecDeque<String>>>,
    sockets: Arc<Mutex<Vec<TcpStream>>>,
) -> thread::Result<std::io::Result<()>> {
    if active_nodes_ips.lock().unwrap().is_empty() {
        println!("ERROR, NO HAY MAS NODOS!");
    } else {
        let mut node_ip = active_nodes_ips.lock().unwrap().pop_front().unwrap();
        loop {
            let stream_result = connect_to_node(&configuracion, &node_ip);
            match stream_result {
                Ok(stream) => {
                    println!("Conectado correctamente a: {:?} \n", node_ip);
                    sockets.lock().unwrap().push(stream);
                    // tomo otra ip y conecto a más nodos
                    if active_nodes_ips.lock().unwrap().is_empty() {
                        println!("ERROR, NO HAY MAS NODOS!");
                        break;
                    }
                    node_ip = active_nodes_ips.lock().unwrap().pop_front().unwrap();
                }
                Err(err) => {
                    println!(
                        "No se pudo conectar a: {:?}, voy a intenar conectarme a otro \n",
                        node_ip
                    );
                    if active_nodes_ips.lock().unwrap().is_empty() {
                        println!("ERROR, NO HAY MAS NODOS!");
                        break;
                    }
                    node_ip = active_nodes_ips.lock().unwrap().pop_front().unwrap();
                }
            };
            //    println!("CANTIDAD SOCKETS: {:?}", sockets.lock().unwrap().len());
        }
    }

    Ok(Ok(()))
}

fn connect_to_node(config: &Config, node_ip: &str) -> Result<TcpStream, Box<dyn Error>> {
    let socket_addr: SocketAddr = node_ip.parse()?;
    let mut stream: TcpStream = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(5))?;
    let local_ip_addr = stream.local_addr()?;
    let version_message = get_version_message(config, socket_addr, local_ip_addr)?;
    version_message.write_to(&mut stream)?;
    let version_response = VersionMessage::read_from(&mut stream)?;
    println!(
        "RECIBO MENSAJE VERSION DEL NODO {:?}: {:?}\n",
        node_ip, version_response
    );
    let verack_message = get_verack_message(config);
    verack_message.write_to(&mut stream)?;
    let verack_response = NonePayloadMessage::read_from(&mut stream)?;
    println!(
        "RECIBO MENSAJE VERACK DEL NODO {:?}: {:?}\n",
        node_ip, verack_response
    );
    Ok(stream)
}

// PASAR AL MODULO QUE CORRESPONDE
fn get_verack_message(config: &Config) -> NonePayloadMessage {
    NonePayloadMessage {
        header: HeaderMessage {
            start_string: config.testnet_start_string,
            command_name: "verack".to_string(),
            payload_size: 0,
            checksum: [0x5d, 0xf6, 0xe0, 0xe2],
        },
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_archivo_configuracion() {}
}
