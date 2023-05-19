use crate::messages::message_header::{read_verack_message, write_verack_message};
use crate::messages::version_message::{get_version_message, VersionMessage};
use std::error::Error;
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::result::Result;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::{fmt, thread};

use crate::config::Config;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum HandShakeError {
    ThreadJoinError(String),
    LockError(String),
    ReadNodeError(String),
    WriteNodeError(String),
    CanNotRead(String),
}

impl fmt::Display for HandShakeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HandShakeError::ThreadJoinError(msg) => write!(f, "ThreadJoinError Error: {}", msg),
            HandShakeError::LockError(msg) => write!(f, "LockError Error: {}", msg),
            HandShakeError::ReadNodeError(msg) => {
                write!(f, "Can not read from socket Error: {}", msg)
            }
            HandShakeError::WriteNodeError(msg) => {
                write!(f, "Can not write in socket Error: {}", msg)
            }
            HandShakeError::CanNotRead(msg) => write!(f, "No more elements in list Error: {}", msg),
        }
    }
}

impl Error for HandShakeError {}
pub struct Handshake;

impl Handshake {
    pub fn handshake(
        config: Arc<Config>,
        active_nodes: &[Ipv4Addr],
    ) -> Result<Vec<TcpStream>, HandShakeError> {
        let lista_nodos = Arc::new(active_nodes);
        let chunk_size = (lista_nodos.len() as f64 / config.n_threads as f64).ceil() as usize;
        let active_nodes_chunks = Arc::new(RwLock::new(
            lista_nodos
                .chunks(chunk_size)
                .map(|chunk| chunk.to_vec())
                .collect::<Vec<_>>(),
        ));
        let sockets = vec![];
        let sockets_lock = Arc::new(RwLock::new(sockets));
        let mut thread_handles = vec![];

        for i in 0..config.n_threads {
            let chunk = active_nodes_chunks
                .write()
                .map_err(|err| HandShakeError::LockError(format!("{}", err)))?[i]
                .clone();
            let configuracion = config.clone();
            let sockets: Arc<RwLock<Vec<TcpStream>>> = Arc::clone(&sockets_lock);
            thread_handles.push(thread::spawn(move || -> Result<(), HandShakeError> {
                conectar_a_nodo(configuracion, sockets, &chunk)?;
                Ok(())
            }));
        }
        println!(
            "{:?}",
            sockets_lock
                .read()
                .map_err(|err| HandShakeError::LockError(format!("{}", err)))?
                .len()
        );
        for handle in thread_handles {
            handle
                .join()
                .map_err(|err| HandShakeError::ThreadJoinError(format!("{:?}", err)))??;
        }
        let sockets = Arc::try_unwrap(sockets_lock)
            .map_err(|err| HandShakeError::LockError(format!("{:?}", err)))?
            .into_inner()
            .map_err(|err| HandShakeError::LockError(format!("{}", err)))?;
        Ok(sockets)
    }
}

// los threads no pueden manejar un dyn Error
// En el libro devuelve thread::Result<std::io::Result<()>>
fn conectar_a_nodo(
    config: Arc<Config>,
    sockets: Arc<RwLock<Vec<TcpStream>>>,
    nodos: &[Ipv4Addr],
) -> Result<(), HandShakeError> {
    for nodo in nodos {
        let configuracion = config.clone();
        match connect_to_node(configuracion, nodo) {
            Ok(stream) => {
                println!("Conectado correctamente a: {:?} \n", nodo);
                sockets
                    .write()
                    .map_err(|err| HandShakeError::LockError(format!("{}", err)))?
                    .push(stream);
            }
            Err(err) => {
                println!(
                    "Error {:?}. No se pudo conectar a: {:?}, voy a intenar conectarme a otro \n",
                    err, nodo
                );
            }
        };
    }
    Ok(())
}

fn connect_to_node(config: Arc<Config>, node_ip: &Ipv4Addr) -> Result<TcpStream, Box<dyn Error>> {
    let socket_addr = SocketAddr::new((*node_ip).into(), config.testnet_port);
    let mut stream: TcpStream =
        TcpStream::connect_timeout(&socket_addr, Duration::from_secs(config.connect_timeout))?;

    let local_ip_addr = stream.local_addr()?;
    let version_message = get_version_message(config, socket_addr, local_ip_addr)?;
    version_message.write_to(&mut stream)?;
    let version_response = VersionMessage::read_from(&mut stream)?;
    println!(
        "RECIBO MENSAJE VERSION DEL NODO {:?}: {:?}\n",
        node_ip, version_response
    );

    write_verack_message(&mut stream)?;
    println!(
        "RECIBO MENSAJE VERACK DEL NODO {:?}: {:?}\n",
        node_ip,
        read_verack_message(&mut stream)
    );
    Ok(stream)
}
