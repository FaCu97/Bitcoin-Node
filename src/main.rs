use bitcoin::config::Config;
use bitcoin::messages::{
    message_header::HeaderMessage,
    none_payload_message::NonePayloadMessage,
    version_message::{get_version_message, VersionMessage},
};
use bitcoin::network::get_active_nodes_from_dns_seed;
use std::error::Error;
use std::net::{SocketAddr, TcpStream};
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

    let sockets: Vec<TcpStream> = handshake(config.clone(), &active_nodes);

    println!("Sockets: {:?}", sockets);
    println!("CANTIDAD SOCKETS: {:?}", sockets.len());
    println!("{:?}", config.user_agent);
    // Acá iría la descarga de los headers
}

fn handshake(config: Config, active_nodes: &[String]) -> Vec<TcpStream> {
    let lista_nodos = Arc::new(active_nodes);
    let chunk_size = (lista_nodos.len() as f64 / config.n_threads as f64).ceil() as usize;
    let active_nodes_chunks = Arc::new(Mutex::new(
        lista_nodos
            .chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect::<Vec<_>>(),
    ));
    let sockets = vec![];
    let sockets_lock = Arc::new(Mutex::new(sockets));
    let mut thread_handles = vec![];

    for i in 0..config.n_threads {
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
fn conectar_a_nodo(
    configuracion: Config,
    sockets: Arc<Mutex<Vec<TcpStream>>>,
    nodos: &[String],
) {
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
