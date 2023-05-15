use bitcoin::config::Config;
use bitcoin::handshake::Handshake;
use bitcoin::initial_block_download::download_headers;
use bitcoin::network::get_active_nodes_from_dns_seed;
use std::env;
use std::error::Error;
//use std::process::exit;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let config: Config =  Config::from(&args)?;
    let active_nodes =  get_active_nodes_from_dns_seed(config.clone())?;
    let mut sockets = Handshake::handshake(config.clone(), &active_nodes);
    println!("Sockets: {:?}", sockets);
    println!("CANTIDAD SOCKETS: {:?}", sockets.len());
    println!("{:?}", config.user_agent);
    // Acá iría la descarga de los headers
    download_headers(&config, &mut sockets)?;
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_archivo_configuracion() {}
}
