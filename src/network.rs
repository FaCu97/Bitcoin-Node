use crate::config::Config;
use std::process::Command;
//const NUMBER_OF_NODES: usize = 8;

pub fn get_active_nodes_from_dns_seed(config: &Config) -> std::io::Result<Vec<String>> {
    let query_reply = Command::new("dig")
        .arg("+short")
        .arg(&config.dns_seed)
        .output()?;
    let active_nodes = String::from_utf8_lossy(&query_reply.stdout);
    let mut nodes_list: Vec<String> = Vec::new();
    for node in active_nodes.lines() {
        let port = &config.testnet_port;
        let mut node_ip = node.to_string();
        node_ip.push(':');
        node_ip.push_str(port); // concateno al final de cada direccion ip ":" + <puerto> (18333)
        nodes_list.push(node_ip);
    }
    if nodes_list.len() < config.number_of_nodes {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No se obtuvieron la cantidad necesaria de nodos de la DNS seed! \n",
        ));
    }

    Ok(nodes_list)
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
