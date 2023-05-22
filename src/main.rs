use bitcoin::block_broadcasting::{BlockBroadcasting, BroadcastingError};
use bitcoin::config::Config;
use bitcoin::handshake::{HandShakeError, Handshake};
use bitcoin::initial_block_download::{initial_block_download, DownloadError};
use bitcoin::logwriter::log_writer::{
    set_up_loggers, shutdown_loggers, write_in_log, LogSender, LoggingError,
};
use bitcoin::network::{get_active_nodes_from_dns_seed, ConnectionToDnsError};
use bitcoin::node::Node;
use bitcoin_hashes::{sha256d, Hash};
//use bitcoin_hashes::hex;
use std::error::Error;
use std::sync::{Arc, RwLock};
use std::{env, fmt};

#[derive(Debug)]
pub enum GenericError {
    DownloadError(DownloadError),
    HandShakeError(HandShakeError),
    ConfigError(Box<dyn Error>),
    ConnectionToDnsError(ConnectionToDnsError),
    LoggingError(LoggingError),
    BroadcastingError(BroadcastingError),
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
            GenericError::LoggingError(msg) => write!(f, "LOGGING ERROR: {}", msg),
            GenericError::BroadcastingError(msg) => write!(f, "BLOCK BROADCASTING ERROR: {}", msg),
        }
    }
}

impl Error for GenericError {}
use hex::{self, ToHex};

fn string_to_bytes(input: &str) -> Result<[u8; 32], hex::FromHexError> {
    let bytes = hex::decode(input)?;
    let mut result = [0; 32];
    result.copy_from_slice(&bytes[..32]);
    Ok(result)
}


fn main() -> Result<(), GenericError> {
    let args: Vec<String> = env::args().collect();
    let config: Arc<Config> = Config::from(&args).map_err(GenericError::ConfigError)?;
    let (
        error_log_sender,
        error_handler,
        info_log_sender,
        info_handler,
        message_log_sender,
        message_handler,
    ) = set_up_loggers(
        config.error_log_path.clone(),
        config.info_log_path.clone(),
        config.message_log_path.clone(),
    )
    .map_err(GenericError::LoggingError)?;
    let logsender = LogSender::new(error_log_sender, info_log_sender, message_log_sender);
    write_in_log(
        logsender.info_log_sender.clone(),
        "Se leyo correctamente el archivo de configuracion\n",
    );
    let active_nodes = get_active_nodes_from_dns_seed(config.clone(), logsender.clone())
        .map_err(GenericError::ConnectionToDnsError)?;
    let sockets = Handshake::handshake(config.clone(), logsender.clone(), &active_nodes)
        .map_err(GenericError::HandShakeError)?;
    // Acá iría la descarga de los headers

    let pointer_to_nodes = Arc::new(RwLock::new(sockets));

    let headers_and_blocks =
        initial_block_download(config, logsender.clone(), pointer_to_nodes.clone()).map_err(
            |err| {
                write_in_log(
                    logsender.error_log_sender.clone(),
                    format!("Error al descargar los bloques: {}", err).as_str(),
                );
                GenericError::DownloadError(err)
            },
        )?;
    let (headers, blocks) = headers_and_blocks;
    let _node = Node {
        headers: headers.clone(),
        block_chain: blocks.clone(),
        utxo_set: vec![],
    };

    //  let headers: Vec<_> = Vec::new();
    //  let blocks: Vec<_> = Vec::new();
    let block_listener = BlockBroadcasting::listen_for_incoming_blocks(
        logsender.clone(),
        pointer_to_nodes,
        Arc::new(RwLock::new(headers)),
        Arc::new(RwLock::new(blocks)),
    )
    .map_err(GenericError::BroadcastingError)?;

    if let Err(err) = handle_input(block_listener) {
        println!("Error al leer la entrada por terminal. {}", err);
    }

    write_in_log(
        logsender.info_log_sender.clone(),
        "TERMINA CORRECTAMENTE EL PROGRAMA!",
    );
    shutdown_loggers(logsender, error_handler, info_handler, message_handler)
        .map_err(GenericError::LoggingError)?;
    Ok(())
}

fn handle_input(block_listener: BlockBroadcasting) -> Result<(), GenericError> {
    loop {
        let mut input = String::new();

        match std::io::stdin().read_line(&mut input) {
            Ok(_) => {
                let command = input.trim();
                if command == "exit" {
                    block_listener
                        .finish()
                        .map_err(GenericError::BroadcastingError)?;
                    break;
                }
            }
            Err(error) => {
                println!("Error al leer la entrada: {}", error);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_archivo_configuracion() {}
}