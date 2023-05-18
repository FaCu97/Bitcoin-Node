use bitcoin::config::Config;
use bitcoin::handshake::{HandShakeError, Handshake};
use bitcoin::initial_block_download::{initial_block_download, DownloadError};
use bitcoin::network::{get_active_nodes_from_dns_seed, ConnectionToDnsError};
use std::error::Error;
use std::{env, fmt};
//use std::process::exit;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub enum GenericError {
    DownloadError(DownloadError),
    HandShakeError(HandShakeError),
    ConfigError(Box<dyn Error>),
    ConnectionToDnsError(ConnectionToDnsError),
}

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GenericError::DownloadError(msg) => write!(f, "DOWNLOAD ERROR: {}", msg),
            GenericError::ConfigError(msg) => write!(f, "CONFIG ERROR: {}", msg),
            GenericError::HandShakeError(msg) => write!(f, "HANDSHAKE ERROR: {}", msg),
            GenericError::ConnectionToDnsError(msg) => {
                write!(f, "CONNECTION TO DNS ERROR: {}", msg)
            }
        }
    }
}

impl Error for GenericError {}

fn main() -> Result<(), GenericError> {
    let args: Vec<String> = env::args().collect();
    let config: Config = Config::from(&args).map_err(GenericError::ConfigError)?;
    let active_nodes = get_active_nodes_from_dns_seed(config.clone())
        .map_err(GenericError::ConnectionToDnsError)?;
    let sockets = Handshake::handshake(config.clone(), &active_nodes)
        .map_err(GenericError::HandShakeError)?;
    println!("Sockets: {:?}", sockets);
    println!("CANTIDAD SOCKETS: {:?}", sockets.len());
    println!("{:?}", config.user_agent);
    // Acá iría la descarga de los headers
    let pointer_to_nodes = Arc::new(RwLock::new(sockets));
    let (headers, blocks) =
        initial_block_download(config, pointer_to_nodes).map_err(GenericError::DownloadError)?;
    println!("DESCARGUE {:?} HEADERS\n", headers.len());
    println!("DESCARGUE {:?} BLOQUES\n", blocks.len());
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_archivo_configuracion() {}
}
