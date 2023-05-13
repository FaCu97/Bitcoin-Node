use crate::messages::message_header::write_verack_message;
use crate::messages::version_message::{get_version_message, VersionMessage};
use std::error::Error;
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::result::Result;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::config::Config;

pub struct Handshake;

impl Handshake {
    pub fn handshake(config: Config, active_nodes: &[Ipv4Addr]) -> Vec<TcpStream> {
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
        }
        Arc::try_unwrap(sockets_lock).unwrap().into_inner().unwrap()
    }
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
    let port: u16 = 18333;
    let socket_addr = SocketAddr::new((*node_ip).into(), port);
    let mut stream: TcpStream = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(5))?;

    let local_ip_addr = stream.local_addr()?;
    let version_message = get_version_message(config, socket_addr, local_ip_addr)?;
    version_message.write_to(&mut stream)?;
    let version_response = VersionMessage::read_from(&mut stream)?;
    println!(
        "RECIBO MENSAJE VERSION DEL NODO {:?}: {:?}\n",
        node_ip, version_response
    );

    let verack_response = write_verack_message(&mut stream)?;
    println!(
        "RECIBO MENSAJE VERACK DEL NODO {:?}: {:?}\n",
        node_ip, verack_response
    );
    Ok(stream)
}
