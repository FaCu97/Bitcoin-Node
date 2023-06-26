use bitcoin::config::Config;
use bitcoin::custom_errors::NodeCustomErrors;
use bitcoin::gtk::interfaz_gtk::Gtk;
use bitcoin::handshake::{HandShakeError, Handshake};
use bitcoin::initial_block_download::{initial_block_download, DownloadError};
use bitcoin::logwriter::log_writer::{
    set_up_loggers, shutdown_loggers, write_in_log, LogSender, LoggingError,
};
use bitcoin::network::{get_active_nodes_from_dns_seed, ConnectionToDnsError};
use bitcoin::node::Node;
use bitcoin::terminal_ui;
use bitcoin::wallet::Wallet;
use std::error::Error;
use std::sync::Arc;
use std::{env, fmt};

#[derive(Debug)]
pub enum GenericError {
    DownloadError(DownloadError),
    HandShakeError(HandShakeError),
    ConfigError(Box<dyn Error>),
    ConnectionToDnsError(ConnectionToDnsError),
    LoggingError(LoggingError),
    NodeHandlerError(NodeCustomErrors),
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
            GenericError::NodeHandlerError(msg) => {
                write!(f, "NODE MESSAGE LISTENER AND WRITER ERROR: {}", msg)
            }
        }
    }
}

impl Error for GenericError {}

fn main() -> Result<(), GenericError> {
    let mut args: Vec<String> = env::args().collect();
    if args.len() == 3 && args[2] == *"-i" {
        Gtk::run();
        // lo saco para que lea config correctamente
        args.pop();
    }
    let config: Arc<Config> = Config::from(&args).map_err(GenericError::ConfigError)?;
    let (
        error_log_sender,
        error_handler,
        info_log_sender,
        info_handler,
        message_log_sender,
        message_handler,
    ) = set_up_loggers(
        config.clone(),
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
    let pointer_to_nodes = Handshake::handshake(config.clone(), logsender.clone(), &active_nodes)
        .map_err(GenericError::HandShakeError)?;
    // Acá iría la descarga de los headers
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
    let node = Node::new(logsender.clone(), pointer_to_nodes, headers, blocks)
        .map_err(GenericError::NodeHandlerError)?;
    let wallet = Wallet::new(node.clone()).map_err(GenericError::NodeHandlerError)?;
    terminal_ui(wallet);
    node.shutdown_node()
        .map_err(GenericError::NodeHandlerError)?;

    shutdown_loggers(logsender, error_handler, info_handler, message_handler)
        .map_err(GenericError::LoggingError)?;

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_archivo_configuracion() {}
}
