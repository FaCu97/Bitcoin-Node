use std::{
    error::Error,
    fmt,
    net::{Ipv4Addr, SocketAddr, ToSocketAddrs},
    sync::Arc,
};

use crate::{
    config::Config,
    logwriter::log_writer::{write_in_log, LogSender},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ConnectionToDnsError(String);

impl fmt::Display for ConnectionToDnsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Can not connect to DNS sedd Error")
    }
}

impl Error for ConnectionToDnsError {}

/// Devuelve una lista de direcciones Ipv4 obtenidas del dns seed
pub fn get_active_nodes_from_dns_seed(
    config: Arc<Config>,
    log_sender: LogSender,
) -> Result<Vec<Ipv4Addr>, ConnectionToDnsError> {
    let mut node_ips = Vec::new();
    let host = config.dns_seed.clone();
    let port = config.net_port;

    let addrs = (host, port)
        .to_socket_addrs()
        .map_err(|err| ConnectionToDnsError(format!("{}", err)))?;

    for addr in addrs {
        if let SocketAddr::V4(v4_addr) = addr {
            node_ips.push(*v4_addr.ip());
        }
    }
    write_in_log(
        log_sender.info_log_sender,
        format!(
            "Se obtuvieron {} ips de la DNS: {:?}\n",
            node_ips.len(),
            node_ips
        )
        .as_str(),
    );
    Ok(node_ips)
}
