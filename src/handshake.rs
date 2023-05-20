use crate::log_writer::{write_in_log, LogSender};
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
        log_sender: LogSender,
        active_nodes: &[Ipv4Addr],
    ) -> Result<Vec<TcpStream>, HandShakeError> {
        write_in_log(log_sender.info_log_sender.clone(), "INICIO DE HANDSHAKE");
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
            let log_sender_clone = log_sender.clone();
            let sockets: Arc<RwLock<Vec<TcpStream>>> = Arc::clone(&sockets_lock);
            thread_handles.push(thread::spawn(move || -> Result<(), HandShakeError> {
                conectar_a_nodo(configuracion, log_sender_clone, sockets, &chunk)?;
                Ok(())
            }));
        }

        for handle in thread_handles {
            handle
                .join()
                .map_err(|err| HandShakeError::ThreadJoinError(format!("{:?}", err)))??;
        }
        let sockets = Arc::try_unwrap(sockets_lock)
            .map_err(|err| HandShakeError::LockError(format!("{:?}", err)))?
            .into_inner()
            .map_err(|err| HandShakeError::LockError(format!("{}", err)))?;
        write_in_log(
            log_sender.info_log_sender.clone(),
            format!("\n{:?} nodos conectados", sockets.len()).as_str(),
        );
        write_in_log(
            log_sender.info_log_sender,
            "Se completo correctamente el handshake\n",
        );
        Ok(sockets)
    }
}

// los threads no pueden manejar un dyn Error
// En el libro devuelve thread::Result<std::io::Result<()>>
fn conectar_a_nodo(
    configuracion: Arc<Config>,
    log_sender: LogSender,
    sockets: Arc<RwLock<Vec<TcpStream>>>,
    nodos: &[Ipv4Addr],
) -> Result<(), HandShakeError> {
    for nodo in nodos {
        match connect_to_node(configuracion.clone(), log_sender.clone(), nodo) {
            Ok(stream) => {
                write_in_log(
                    log_sender.info_log_sender.clone(),
                    format!("Conectado correctamente a: {:?}", nodo).as_str(),
                );
                sockets
                    .write()
                    .map_err(|err| HandShakeError::LockError(format!("{}", err)))?
                    .push(stream);
            }
            Err(err) => {
                write_in_log(log_sender.error_log_sender.clone(),format!("No se pudo conectar al nodo: {:?}, voy a intenar conectarme a otro. Error {:?}.", nodo, err).as_str());
            }
        };
    }
    Ok(())
}

fn connect_to_node(
    config: Arc<Config>,
    log_sender: LogSender,
    node_ip: &Ipv4Addr,
) -> Result<TcpStream, Box<dyn Error>> {
    let socket_addr = SocketAddr::new((*node_ip).into(), config.testnet_port);
    let mut stream: TcpStream =
        TcpStream::connect_timeout(&socket_addr, Duration::from_secs(config.connect_timeout))?;

    let local_ip_addr = stream.local_addr()?;
    let version_message = get_version_message(config, socket_addr, local_ip_addr)?;
    version_message.write_to(&mut stream)?;
    VersionMessage::read_from(log_sender.clone(), &mut stream)?;
    write_verack_message(&mut stream)?;
    read_verack_message(log_sender, &mut stream)?;
    Ok(stream)
}
