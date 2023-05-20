use bitcoin::block_broadcasting::BlockBroadcasting;
use bitcoin::config::Config;
use bitcoin::handshake::{HandShakeError, Handshake};
use bitcoin::initial_block_download::{initial_block_download, DownloadError};
use bitcoin::log_writer::{
    set_up_loggers, shutdown_loggers, write_in_log, LogSender, LoggingError,
};
use bitcoin::network::{get_active_nodes_from_dns_seed, ConnectionToDnsError};
use bitcoin::node::Node;
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
        }
    }
}

impl Error for GenericError {}

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
        match initial_block_download(config, logsender.clone(), pointer_to_nodes.clone()) {
            Ok(headers_and_blocks) => headers_and_blocks,
            Err(err) => {
                write_in_log(
                    logsender.error_log_sender,
                    format!("Error al descargar los bloques: {}", err).as_str(),
                );
                return Err(GenericError::DownloadError(err));
            }
        };
    let (headers, blocks) = headers_and_blocks;
    write_in_log(
        logsender.info_log_sender.clone(),
        format!("TOTAL DE HEADERS DESCARGADOS: {}", headers.len()).as_str(),
    );
    write_in_log(
        logsender.info_log_sender.clone(),
        format!("TOTAL DE BLOQUES DESCARGADOS: {}\n", blocks.len()).as_str(),
    );
    
    let block_listener = BlockBroadcasting::listen_for_incoming_blocks(
        logsender.clone(),
        pointer_to_nodes,
        headers.clone(),
        blocks.clone(),
    );
    let _node = Node {
        headers,
        block_chain: blocks,
        utxo_set: vec![],
    };

    loop {
        let mut input = String::new();
        
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => {
                let command = input.trim();
                if command == "exit" {
                    block_listener.finish().unwrap();
                    break;
                }
                // Resto del código para manejar el comando ingresado
                println!("Comando ingresado: {}", command);
            }
            Err(error) => {
                println!("Error al leer la entrada: {}", error);
            }
        }
    }

    write_in_log(
        logsender.info_log_sender.clone(),
        "TERMINA CORRECTAMENTE EL PROGRAMA!",
    );
    shutdown_loggers(logsender, error_handler, info_handler, message_handler)
        .map_err(GenericError::LoggingError)?;
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_archivo_configuracion() {}
}
