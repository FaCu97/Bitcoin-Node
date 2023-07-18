use crate::custom_errors::NodeCustomErrors;
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

use crate::config::Config;

/// Realiza el handshake con todos los nodos de la lista recibida por parámetro.
/// Devuelve un vector de sockets con los nodos con los que se pudo establecer la conexión.
/// En caso de no poder conectarse a ninguno, devuelve un error.
pub fn handshake_with_nodes(
    config: &Arc<Config>,
    log_sender: &LogSender,
    active_nodes: Vec<Ipv4Addr>,
) -> Result<Arc<RwLock<Vec<TcpStream>>>, NodeCustomErrors> {
    write_in_log(&log_sender.info_log_sender, "INICIO DE HANDSHAKE");
    println!("Realizando hadshake con nodos...{:?}\n", active_nodes);
    let sockets = vec![];
    let pointer_to_sockets = Arc::new(RwLock::new(sockets));
    connect_to_nodes(
        config,
        log_sender,
        pointer_to_sockets.clone(),
        &active_nodes,
    )?;
    let amount_of_ips = pointer_to_sockets
        .read()
        .map_err(|err| NodeCustomErrors::LockError(format!("{:?}", err)))?
        .len();
    write_in_log(
        &log_sender.info_log_sender,
        format!("{:?} nodos conectados", amount_of_ips).as_str(),
    );
    write_in_log(
        &log_sender.info_log_sender,
        "Se completo correctamente el handshake\n",
    );
    Ok(pointer_to_sockets)
}

/// Realiza la conexión con todos los nodos de la lista recibida por parámetro.
/// Guarda el los mismos en la lista de sockets recibida.
/// En caso de no poder conectarse, continua intentando con el siguiente.
fn connect_to_nodes(
    config: &Arc<Config>,
    log_sender: &LogSender,
    sockets: Arc<RwLock<Vec<TcpStream>>>,
    nodes: &[Ipv4Addr],
) -> Result<(), NodeCustomErrors> {
    for node in nodes {
        match connect_to_node(config, log_sender, node) {
            Ok(stream) => {
                write_in_log(
                    &log_sender.info_log_sender,
                    format!("Conectado correctamente a: {:?}", node).as_str(),
                );
                sockets
                    .write()
                    .map_err(|err| NodeCustomErrors::LockError(format!("{}", err)))?
                    .push(stream);
            }
            Err(err) => {
                write_in_log(
                    &log_sender.error_log_sender,
                    format!("No se pudo conectar al nodo: {:?}. Error {:?}.", node, err).as_str(),
                );
            }
        };
    }
    // si no se pudo conectar a ningun nodo devuelvo error
    if sockets
        .read()
        .map_err(|err| NodeCustomErrors::LockError(format!("{}", err)))?
        .is_empty()
    {
        return Err(NodeCustomErrors::HandshakeError(
            "No se pudo conectar a ningun nodo".to_string(),
        ));
    }
    Ok(())
}

/// Realiza la conexión con un nodo.
/// Envía y recibe los mensajes necesarios para establecer la conexión
/// Devuelve el socket o un error
fn connect_to_node(
    config: &Arc<Config>,
    log_sender: &LogSender,
    node_ip: &Ipv4Addr,
) -> Result<TcpStream, Box<dyn Error>> {
    let socket_addr = SocketAddr::new((*node_ip).into(), config.net_port);
    let mut stream: TcpStream =
        TcpStream::connect_timeout(&socket_addr, Duration::from_secs(config.connect_timeout))?;
    let local_ip_addr = stream.local_addr()?;
    let version_message = get_version_message(config, socket_addr, local_ip_addr)?;
    version_message.write_to(&mut stream)?;
    VersionMessage::read_from(log_sender, &mut stream)?;
    write_verack_message(&mut stream)?;
    read_verack_message(log_sender, &mut stream)?;
    write_sendheaders_message(&mut stream)?;
    Ok(stream)
}
