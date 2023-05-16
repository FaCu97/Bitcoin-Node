use bitcoin::config::Config;
use bitcoin::handshake::Handshake;
use bitcoin::initial_block_download::ibd;
use bitcoin::network::get_active_nodes_from_dns_seed;
use std::env;
use std::error::Error;
//use std::process::exit;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let config: Config =  Config::from(&args)?;
    let active_nodes =  get_active_nodes_from_dns_seed(config.clone())?;
    let sockets = Handshake::handshake(config.clone(), &active_nodes);
    println!("Sockets: {:?}", sockets);
    println!("CANTIDAD SOCKETS: {:?}", sockets.len());
    println!("{:?}", config.user_agent);
    // Acá iría la descarga de los headers
    let pointer_to_nodes = Arc::new(Mutex::new(sockets));
    ibd(config, pointer_to_nodes)?;
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_archivo_configuracion() {}
}
