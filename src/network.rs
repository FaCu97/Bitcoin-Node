use std::process::Command;
const NUMBER_OF_NODES: usize = 8;

pub fn get_active_nodes_from_dns_seed(dns_seed: String) -> std::io::Result<Vec<String>> {
    let query_reply = Command::new("dig").arg("+short").arg(dns_seed).output()?;
    let active_nodes = String::from_utf8_lossy(&query_reply.stdout);
    let mut nodes_list = vec![];
    for node in active_nodes.lines().take(NUMBER_OF_NODES) {
        nodes_list.push(node.to_string());
    }
    if nodes_list.len() < NUMBER_OF_NODES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No se obtuvieron la cantidad necesaria de nodos de la DNS seed! \n",
        ));
    }
    Ok(nodes_list)
}

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
