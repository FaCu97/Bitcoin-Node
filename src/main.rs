use bitcoin::config::Config;
use bitcoin::handshake::Handshake;

use bitcoin::network::get_active_nodes_from_dns_seed;
use std::env;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config: Config = match Config::from(&args) {
        Err(e) => {
            println!("Application error: {e}");
            exit(1)
        }
        Ok(config) => config,
    };
    let active_nodes = match get_active_nodes_from_dns_seed(config.clone()) {
        Err(e) => {
            println!("ERROR: {}", e);
            exit(-1)
        }
        Ok(active_nodes) => active_nodes,
    };

    let sockets = Handshake::handshake(config.clone(), &active_nodes);

    println!("Sockets: {:?}", sockets);
    println!("CANTIDAD SOCKETS: {:?}", sockets.len());
    println!("{:?}", config.user_agent);
    // Acá iría la descarga de los headers
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_archivo_configuracion() {}
}
