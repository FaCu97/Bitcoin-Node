use bitcoin::config::Config;
use bitcoin::gtk::gtk::Gtk;
use bitcoin::handler::node_message_handler::NodeMessageHandlerError;
use bitcoin::handshake::{HandShakeError, Handshake};
use bitcoin::initial_block_download::{initial_block_download, DownloadError};
use bitcoin::logwriter::log_writer::{
    set_up_loggers, shutdown_loggers, write_in_log, LogSender, LoggingError,
};
use bitcoin::network::{get_active_nodes_from_dns_seed, ConnectionToDnsError};
use bitcoin::node::Node;
use bitcoin::wallet::Wallet;
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
    NodeHandlerError(NodeMessageHandlerError),
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

    let node = Node::new(logsender.clone(), pointer_to_nodes, headers, blocks)
        .map_err(GenericError::NodeHandlerError)?;
    //  let headers: Vec<_> = Vec::new();
    //  let blocks: Vec<_> = Vec::new();
    //let wallet = wallet::Wallet { account: vec![User{private_key: "cTJdkwZ1JScFHVHMR26XLzcbu8n5yWpTZLKkx4LnV8mJRpTGfawQ".to_string(), address: "mnzKX6goXp4xNwxKDFr8LHnPsJcRdqgAGY".to_string(), pending_transactions: vec![]}], node };
    let mut wallet = Wallet::new(node.clone()).map_err(GenericError::NodeHandlerError)?;
    wallet
        .add_account(
            "cSqmqW48wCeoUF8FCJvVsqUGwcvir27bKWCFj1MTFszFdn2Dduim".to_string(),
            "mocD12x6BV3qK71FwG98h5VWZ4qVsbaoi9".to_string(),
        )
        .map_err(GenericError::NodeHandlerError)?;
    wallet
        .add_account(
            "cSVpNr93PCFhizA9ELgnmkwRxycL1bn6vx1WBJ7SmE8ve9Aq1PzZ".to_string(),
            "mmkNBGEEzj7ePpDii91zgUXi3i3Hgkpi9a".to_string(),
        )
        .map_err(GenericError::NodeHandlerError)?;

    match wallet.make_transaction_index(0, "mmkNBGEEzj7ePpDii91zgUXi3i3Hgkpi9a", 10, 10 as i64) {
        Ok(_) => println!("Transaccion ok"),
        Err(e) => println!("Error al realizar la transaccion: {}", e),
    }

    /*
    let wallet = Wallet {
        node,
        current_account_index: 0,
        //accounts: vec![Account {private_key:"cTJdkwZ1JScFHVHMR26XLzcbu8n5yWpTZLKkx4LnV8mJRpTGfawQ".to_string(),address:"mnzKX6goXp4xNwxKDFr8LHnPsJcRdqgAGY".to_string(),utxo_set:vec![], pending_transactions: Arc::new(RwLock::new(Vec::new())) }]
        /*
        accounts: vec![Account {
            private_key: "cSVpNr93PCFhizA9ELgnmkwRxycL1bn6vx1WBJ7SmE8ve9Aq1PzZ".to_string(),
            address: "mmkNBGEEzj7ePpDii91zgUXi3i3Hgkpi9a".to_string(),
            utxo_set: vec![],
            pending_transactions: Arc::new(RwLock::new(Vec::new())),
        }],
        */
        accounts: vec![Account {
            private_key: "cSqmqW48wCeoUF8FCJvVsqUGwcvir27bKWCFj1MTFszFdn2Dduim".to_string(),
            address: "mocD12x6BV3qK71FwG98h5VWZ4qVsbaoi9".to_string(),
            utxo_set: vec![],
            pending_transactions: Arc::new(RwLock::new(Vec::new())),
        }],
    };
    */

    if let Err(err) = handle_input(node) {
        println!("Error al leer la entrada por terminal. {}", err);
    }

    // esta parte es para explicar el comportamiento en la demo !!

    // mostrar_comportamiento_del_nodo(node);/*

    /*let block_1 = node.block_chain.read().unwrap()[0].clone();
    let block_2 = node.block_chain.read().unwrap()[1].clone();
    let mut hash_block_1 = block_1.block_header.hash();
    hash_block_1.reverse();
    let block1_hex: String = hash_block_1.encode_hex::<String>();
    println!("bloque 1 :{}", block1_hex);
    let mut hash_block_2 = block_2.block_header.hash();
    hash_block_2.reverse();
    let block2_hex: String = hash_block_2.encode_hex::<String>();
    println!("bloque 2 :{}", block2_hex);

    let height_block = block_1.txn[0].tx_in[0].height.clone().unwrap();
    let height_hex: String = height_block.encode_hex::<String>();
    println!("height :{}", height_hex);
    let height_block = block_2.txn[0].tx_in[0].height.clone().unwrap();
    let height_hex: String = height_block.encode_hex::<String>();
    println!("height :{}", height_hex);
    */

    write_in_log(
        logsender.info_log_sender.clone(),
        "TERMINA CORRECTAMENTE EL PROGRAMA!",
    );
    shutdown_loggers(logsender, error_handler, info_handler, message_handler)
        .map_err(GenericError::LoggingError)?;

    Ok(())
}

fn handle_input(node: Node) -> Result<(), GenericError> {
    loop {
        let mut input = String::new();

        match std::io::stdin().read_line(&mut input) {
            Ok(_) => {
                let command = input.trim();
                if command == "exit" {
                    node.shutdown_node()
                        .map_err(GenericError::NodeHandlerError)?;
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
/*
fn mostrar_comportamiento_del_nodo(node: Node) {
    let mut header_1 = node.headers[0].hash();
    header_1.reverse();
    let mut header_2 = node.headers[1].hash();
    header_2.reverse();
    let header_1_hex = header_1.encode_hex::<String>();
    let header_2_hex = header_2.encode_hex::<String>();
    println!("header 1 : {}", header_1_hex);
    println!("header 2 : {}", header_2_hex);

    let mut bloque_1 = node.block_chain[0].block_header.hash();
    bloque_1.reverse();
    let bloque1_hex: String = bloque_1.encode_hex::<String>();
    let validate = node.block_chain[0].validate();
    println!("validate devuelve: {}, {}", validate.0, validate.1);
    println!("bloque : {}", bloque1_hex);
    println!(
        "cantidad de transacciones en el bloque : {}",
        node.block_chain[0].txn_count.decoded_value()
    );
    println!(
        "version del bloque : {:x}",
        node.block_chain[0].block_header.version
    );
    println!(
        "nbits del bloque : {:x}",
        node.block_chain[0].block_header.n_bits
    );
    println!(
        "nonce del bloque : {:x}",
        node.block_chain[0].block_header.nonce
    );
    let transaccion = &node.block_chain[0].txn[0];
    let mut hash = transaccion.hash();
    hash.reverse();
    let hash_hex: String = hash.encode_hex::<String>();
    println!("hash de la primera transaccion : {}", hash_hex);
    println!("version de la transaccion : {}", transaccion.version);
    println!(
        "inputs de la transaccion : {}",
        transaccion.txin_count.decoded_value()
    );
    println!(
        "outputs de la transaccion : {}",
        transaccion.txout_count.decoded_value()
    );
    println!("lock time de la transaccion : {}", transaccion.lock_time);
}*/

#[cfg(test)]
mod tests {

    #[test]
    fn test_archivo_configuracion() {}
}
