use bitcoin::config::Config;
use bitcoin::handshake::Handshake;
use bitcoin::initial_block_download::{ibd, DownloadError};
use bitcoin::network::get_active_nodes_from_dns_seed;
use std::error::Error;
use std::{env, fmt};
//use std::process::exit;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum GenericError {
    DownloadError(DownloadError),
    HandShakeError(Box<dyn Error>),
    ConfigError(Box<dyn Error>),
}

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GenericError::DownloadError(msg) => write!(f, "DOWNLOAD ERROR: {}", msg),
            GenericError::ConfigError(msg) => write!(f, "CONFIG ERROR: {}", msg),
            GenericError::HandShakeError(msg) => write!(f, "HANDSHAKE ERROR: {}", msg),
        }
    }
}

impl Error for GenericError {}

fn main() -> Result<(), GenericError> {
    let args: Vec<String> = env::args().collect();
    let config: Config = Config::from(&args).map_err(|err| GenericError::ConfigError(err))?;
    let active_nodes = get_active_nodes_from_dns_seed(config.clone())
        .map_err(|err| GenericError::HandShakeError(Box::new(err)))?;
    let sockets = Handshake::handshake(config.clone(), &active_nodes);
    println!("Sockets: {:?}", sockets);
    println!("CANTIDAD SOCKETS: {:?}", sockets.len());
    println!("{:?}", config.user_agent);
    // Acá iría la descarga de los headers
    let pointer_to_nodes = Arc::new(Mutex::new(sockets));
    ibd(config, pointer_to_nodes).map_err(|err| GenericError::DownloadError(err))?;
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_archivo_configuracion() {}
}
