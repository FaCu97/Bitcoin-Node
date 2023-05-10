use bitcoin::compact_size_uint::CompactSizeUint;
use bitcoin::config::Config;
use bitcoin::messages::{
    message_header::{get_checksum, HeaderMessage},
    none_payload_message::NonePayloadMessage,
    version_message::VersionMessage,
    version_payload::{get_current_unix_epoch_time, get_ipv6_address_ip, VersionPayload},
};
use bitcoin::network::get_active_nodes_from_dns_seed;
use rand::Rng;
use std::collections::VecDeque;
use std::error::Error;
use std::net::{SocketAddr, TcpStream};
use std::process::exit;
use std::result::Result;
use std::sync::{Arc, Mutex};
use std::{env, thread};

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = match Config::from(&args) {
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
    
    /* código para testear
    println!("ACA1:\n");
    let result = connect_to_node(&config, &"213.22.192.145:18333");
    match result {
        Ok(_) => println!("OK:\n"),
        Err(e) => println!("Error:\n"),
    }
    println!("ACA:\n");
    */




    println!("{:?}", active_nodes);
  //  println!("{:?}", &config);
    let mut nodo_prueba:VecDeque<String> = VecDeque::new();
    /* 
    nodo_prueba.push_front("213.22.192.145:18333".to_string());
    */
    let active_nodes_lock = Arc::new(Mutex::new(active_nodes));
    
    let configuracion_lock = Arc::new(config);
       let active_nodes_lock_ref = active_nodes_lock.clone();
    let mut sockets: Vec<TcpStream> = Vec::new();
    let sockets_lock = Arc::new(Mutex::new(sockets));
    let NTHREADS = 8; // pasar a Config
    for _ in 0..NTHREADS {
        let active_nodes = Arc::clone(&active_nodes_lock);
        let configuracion = Arc::clone(&configuracion_lock);
        let sockets: Arc<Mutex<Vec<TcpStream>>> = Arc::clone(&sockets_lock);
        let mut thread_handles = vec![];
        thread_handles.push(thread::spawn(move|| {
            conectar_a_nodo(configuracion, active_nodes, sockets)
        }));
        for handle in thread_handles {
            handle.join().unwrap();
        }
    }
    println!("{:?}", sockets_lock.lock().unwrap().len());

    // Acá iría la descarga de los headers
}

// los threads no pueden manejar un dyn Error
// En el libro devuelve thread::Result<std::io::Result<()>>
fn conectar_a_nodo(
    configuracion: Arc<Config>,
    active_nodes_ips: Arc<Mutex<VecDeque<String>>>,
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
            println!("CANTIDAD SOCKETS: {:?}", sockets.lock().unwrap().len());

        }
    }

    Ok(Ok(()))
}

fn connect_to_node(config: &Config, node_ip: &str) -> Result<TcpStream, Box<dyn Error>> {
    let socket_addr: SocketAddr = node_ip.parse()?;
        let mut stream: TcpStream = TcpStream::connect(socket_addr)?;
    // con la ip 213.22.192.145:18333 no termina de conectar nunca, se cuelga en la linea de arriba (capaz tarda mucho en resolver que no se puede conectar)
    let local_ip_addr = stream.local_addr()?;
    let version_message = get_version_message(config, socket_addr, local_ip_addr)?;
    version_message.write_to(&mut stream)?;
    let v = VersionMessage::read_from(&mut stream)?;
    println!("ME DEVUELVE MENSAJE VERSION: {:?}\n", v);
    let verack_message = get_verack_message(config);
    verack_message.write_to(&mut stream)?;
    let ve = NonePayloadMessage::read_from(&mut stream)?;
    println!("ME DEVUELVE MENSAJE VERACK: {:?}\n", ve);
    Ok(stream)
}

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

fn get_version_payload(
    config: &Config,
    socket_addr: SocketAddr,
    local_ip_addr: SocketAddr,
) -> Result<VersionPayload, Box<dyn Error>> {
    let timestamp: i64 = get_current_unix_epoch_time()?;
    Ok(VersionPayload {
        version: config.protocol_version,
        services: 0u64,
        timestamp,
        addr_recv_service: 1u64,
        addr_recv_ip: get_ipv6_address_ip(socket_addr),
        addr_recv_port: 18333,
        addr_trans_service: 0u64,
        addr_trans_ip: get_ipv6_address_ip(local_ip_addr),
        addr_trans_port: 18333,
        nonce: rand::thread_rng().gen(),
        user_agent_bytes: CompactSizeUint::new(16u128),
        user_agent: config.user_agent.to_string(),
        start_height: 1,
        relay: true,
    })
}
fn get_version_message(
    config: &Config,
    socket_addr: SocketAddr,
    local_ip_addr: SocketAddr,
) -> Result<VersionMessage, Box<dyn Error>> {
    let version_payload = get_version_payload(config, socket_addr, local_ip_addr)?;
    let version_header = HeaderMessage {
        start_string: config.testnet_start_string,
        command_name: "version".to_string(),
        payload_size: version_payload.to_le_bytes().len() as u32,
        checksum: get_checksum(&version_payload.to_le_bytes()),
    };
    Ok(VersionMessage {
        header: version_header,
        payload: version_payload,
    })
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_archivo_configuracion() {}
}
