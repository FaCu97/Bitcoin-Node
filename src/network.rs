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
    let port = config.dns_port;

    let addrs = (host, port)
        .to_socket_addrs()
        .map_err(|err| ConnectionToDnsError(format!("{}", err)))?;
    /*
        node_ips.push(Ipv4Addr::new(94, 130,58, 119));
        node_ips.push(Ipv4Addr::new(94, 130,58, 119));
        node_ips.push(Ipv4Addr::new(54, 251,158, 5));
        node_ips.push(Ipv4Addr::new(144, 126,221, 254));
        node_ips.push(Ipv4Addr::new(94, 130, 58, 119));
        node_ips.push(Ipv4Addr::new(185, 21, 217, 48));
        node_ips.push(Ipv4Addr::new(94, 130, 58, 119));
        node_ips.push(Ipv4Addr::new(185, 21, 217, 48));
        node_ips.push(Ipv4Addr::new(185, 21, 217, 48));
        node_ips.push(Ipv4Addr::new(185, 21, 217, 48));
    */
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

/*
#[cfg(test)]
mod tests {
    use super::*;
    fn is_valid_ip_address(ip_address: &str) -> bool {
        let octets: Vec<&str> = ip_address.split('.').collect();

        if octets.len() != 4 {
            return false;
        }

        for octet in octets {
            match octet.parse::<u8>() {
                Ok(_) => continue,
                _ => return false,
            }
        }

        true
    }
    #[test]
    fn getting_number_of_nodes_correctly_with_one_valid_dns_seed_direction() {
        let valid_dns = "seed.testnet.bitcoin.sprovoost.nl".to_string();
        let active_nodes = get_active_nodes_from_dns_seed(valid_dns).unwrap();
        assert_eq!(NUMBER_OF_NODES, active_nodes.len())
    }
    #[test]
    fn getting_number_of_nodes_correctly_with_other_valid_dns_seed_direction() {
        let valid_dns = "testnet-seed.bitcoin.jonasschnelli.ch".to_string();
        let active_nodes = get_active_nodes_from_dns_seed(valid_dns).unwrap();
        assert_eq!(NUMBER_OF_NODES, active_nodes.len())
    }
    #[test]
    fn getting_ip_addresses_correctly_with_valid_dns_seed_direction() {
        let valid_dns = "testnet-seed.bitcoin.jonasschnelli.ch".to_string();
        let active_nodes = get_active_nodes_from_dns_seed(valid_dns).unwrap();
        for node in active_nodes {
            assert!(is_valid_ip_address(node.as_str()))
        }
    }
    #[test]
    fn getting_error_with_invalid_dns_seed_direction() {
        let invalid_dns = "invalid_dns_seed".to_string();
        let active_nodes = get_active_nodes_from_dns_seed(invalid_dns);
        assert!(active_nodes.is_err());
        assert_eq!(
            active_nodes.unwrap_err().to_string(),
            "No se obtuvieron la cantidad necesaria de nodos de la DNS seed! \n"
        )
    }
}
*/
