use std::{env, thread};
use bitcoin::config::Config;
use bitcoin::compact_size_uint::CompactSizeUint;
use std::process::exit;
use std::sync::{Arc, Mutex};
use bitcoin::messages::{message_header::{HeaderMessage, get_checksum},version_payload::{get_ipv6_address_ip, get_current_unix_epoch_time, VersionPayload},version_message::VersionMessage};
use bitcoin::network::{get_active_nodes_from_dns_seed};
use rand::Rng;
use std::result::Result;
use std::net::{SocketAddr, TcpStream};
//use std::io::Error;
use std::error::Error;
const START_STRING: [u8; 4] = [0x0b, 0x11, 0x09, 0x07];

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = match Config::from(&args) {
        Err(e) => {
            println!("Application error: {e}");
            exit(1)
        }
        Ok(config) => config
    };

    let active_nodes =
        match get_active_nodes_from_dns_seed(&config) {
            Err(e) => {
                println!("ERROR: {}", e);
                exit(-1)
            }
            Ok(active_nodes) => active_nodes,
        };
    
    println!("{:?}", active_nodes);
    let active_nodes_lock = Arc::new(Mutex::new(active_nodes));
 //   let active_nodes_lock_ref = active_nodes_lock.clone();

    for _ in 0..8 {
        let active_nodes = Arc::clone(&active_nodes_lock);
        let handle = thread::spawn(move || {
            if active_nodes.lock().unwrap().is_empty() {
                println!("ERROR, NO HAY MAS NODOS!");
            } else {
                let mut node_ip = active_nodes.lock().unwrap().pop_front().unwrap();
                loop{
                    let stream = connect_to_node(&node_ip);
                    if stream.is_ok(){
                        println!("Conectado correctamente a: {:?} \n", node_ip);
                        break;
                    }
                    else{
                        println!("No se pudo conectar a: {:?}, voy a intenar conectarme a otro \n", node_ip);
                        if active_nodes.lock().unwrap().is_empty() {
                            println!("ERROR, NO HAY MAS NODOS!");
                            break;
                        }
                        node_ip = active_nodes.lock().unwrap().pop_front().unwrap();
                    }
                }
            }
        });
        handle.join().unwrap();
    }

}





fn connect_to_node(config:&Config, node_ip: &String) -> Result<TcpStream, Box<dyn Error>>{
    let socket_addr: SocketAddr =  node_ip.parse()?;
    let mut stream: TcpStream = TcpStream::connect(socket_addr)?;
    let local_ip_addr = stream.local_addr()?;
    let version_message = get_version_message(config, socket_addr, local_ip_addr)?;
    version_message.write_to(&mut stream)?;
    VersionMessage::read_from(&mut stream)?;
    let verack_message = get_verack_message();
    verack_message.write_to(&mut stream)?;
    VerackMessage::read_from(&mut stream)?;
    Ok(stream)
}


fn get_verack_message() -> VerackMessage {
    VerackMessage {
        header: HeaderMessage {
            start_string: START_STRING,
            command_name: "verack".to_string(),
            payload_size: 0,
            checksum: EMPTY_CHECKSUM,
        },
    }
}

fn get_version_payload(config:&Config, socket_addr: SocketAddr, local_ip_addr: SocketAddr) -> Result<VersionPayload, Box<dyn Error>> {
    let timestamp: i64 = get_current_unix_epoch_time()?;
    Ok(VersionPayload {
        version: config.PROTOCOL_VERSION,
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
        user_agent: config.USER_AGENT.to_string(),
        start_height: 1,
        relay: true,
    })
}
fn get_version_message(config:&Config, socket_addr: SocketAddr, local_ip_addr: SocketAddr) -> Result<VersionMessage, Box<dyn Error>> {
    let version_payload = get_version_payload(config, socket_addr, local_ip_addr)?;
    let version_header = HeaderMessage {
        start_string: START_STRING,
        command_name: "version".to_string(),
        payload_size: version_payload.to_le_bytes().len() as u32,
        checksum: get_checksum(&version_payload.to_le_bytes()),
    };
    Ok(VersionMessage {
        header: version_header,
        payload: version_payload,
    })
}
/* 


*/



#[cfg(test)]
mod tests {

    #[test]
    fn test_archivo_configuracion() {}
}
