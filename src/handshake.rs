use crate::logwriter::log_writer::{write_in_log, LogSender};
use crate::messages::message_header::{
    read_verack_message, write_sendheaders_message, write_verack_message,
};
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
    /// Realiza la conexión a los nodos con múltiples threads
    /// Recibe las direcciones IP de los nodos.
    /// Devuelve un vector de sockets o un error si no se pudo completar.
    pub fn handshake(
        config: Arc<Config>,
        log_sender: LogSender,
        active_nodes: &[Ipv4Addr],
    ) -> Result<Arc<RwLock<Vec<TcpStream>>>, HandShakeError> {
        write_in_log(&log_sender.info_log_sender, "INICIO DE HANDSHAKE");
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
            thread_handles.push(thread::spawn(move || {
                connect_to_nodes(configuracion, log_sender_clone, sockets, &chunk)
            }));
        }

        for handle in thread_handles {
            handle
                .join()
                .map_err(|err| HandShakeError::ThreadJoinError(format!("{:?}", err)))??;
        }
        let cantidad_sockets = sockets_lock
            .read()
            .map_err(|err| HandShakeError::LockError(format!("{:?}", err)))?
            .len();

        write_in_log(
            &log_sender.info_log_sender,
            format!("{:?} nodos conectados", cantidad_sockets).as_str(),
        );
        write_in_log(
            &log_sender.info_log_sender,
            "Se completo correctamente el handshake\n",
        );
        Ok(sockets_lock)
    }
}

/// Realiza la conexión con todos los nodos de la lista recibida por parámetro.
/// Guarda el los mismos en la lista de sockets recibida.
/// En caso de no poder conectarse, continua intentando con el siguiente.
fn connect_to_nodes(
    configuracion: Arc<Config>,
    log_sender: LogSender,
    sockets: Arc<RwLock<Vec<TcpStream>>>,
    nodos: &[Ipv4Addr],
) -> Result<(), HandShakeError> {
    for nodo in nodos {
        match connect_to_node(configuracion.clone(), log_sender.clone(), nodo) {
            Ok(stream) => {
                write_in_log(
                    &log_sender.info_log_sender,
                    format!("Conectado correctamente a: {:?}", nodo).as_str(),
                );
                sockets
                    .write()
                    .map_err(|err| HandShakeError::LockError(format!("{}", err)))?
                    .push(stream);
            }
            Err(err) => {
                write_in_log(&log_sender.error_log_sender,format!("No se pudo conectar al nodo: {:?}, voy a intenar conectarme a otro. Error {:?}.", nodo, err).as_str());
            }
        };
    }
    Ok(())
}

/// Realiza la conexión con un nodo.
/// Envía y recibe los mensajes necesarios para establecer la conexión
/// De vuelve el socket o un error
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
    write_sendheaders_message(&mut stream)?;
    Ok(stream)
}
