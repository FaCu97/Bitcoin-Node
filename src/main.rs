use bitcoin::block::Block;
use bitcoin::block_header::BlockHeader;
//use bitcoin::block_broadcasting::listen_for_incoming_blocks;
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
    /*
    listen_for_incoming_blocks(
        logsender.clone(),
        pointer_to_nodes,
        headers.clone(),
        blocks.clone(),
    );*/
    //println!("SALI DE LA FUNCION!!\n");
    let node = Node {
        headers,
        block_chain: blocks,
        utxo_set: vec![],
    }; 
    let validate = node.block_chain[0].validate();
    let root = node.block_chain[0].generate_merkle_root();
    let merkle = node.block_chain[0].block_header.merkle_root_hash;
    let hex_root = root.encode_hex::<String>();
    let hex_merkle = merkle.encode_hex::<String>();

    println!("{}",validate.0);
    println!("esperada: {}",hex_merkle);
    println!("nuestra: {}",hex_root);
    //let mut transaction = node.block_chain[0].txn[1].hash();
    let aux= &node.block_chain[0].txn[0];
    let mut coinbase_transaction = aux.hash();
    coinbase_transaction.reverse();
    
    let hex_tx = coinbase_transaction.encode_hex::<String>();
    let version = aux.version;
    let txin_count = aux.txin_count.decoded_value();
    let txout_count = aux.txout_count.decoded_value();
    let lock_time = aux.lock_time;
    let mut block_header = node.block_chain[0].block_header.hash();
    block_header.reverse();
    let tx_count = node.block_chain[0].txn_count.decoded_value();
    let hex_hdr = block_header.encode_hex::<String>();
    let mut  header = node.headers[0].hash();
    header.reverse();
    let aux_1 = *sha256d::Hash::hash(&header).as_byte_array();
    let aux_string = aux_1.encode_hex::<String>();

    let hex_string = header.encode_hex::<String>();
    let mut bytes =Vec::new(); 
    aux.marshalling(& mut bytes);
    let mut coin_hash = *sha256d::Hash::hash(&bytes).as_byte_array();
    coin_hash.reverse();
    let coin_string = coin_hash.encode_hex::<String>();
    println!("header del bloque  : {}",hex_hdr);
/* 
    println!(" el header : {}",hex_string);
    println!("transaction del bloque  : {}",hex_tx);
    println!("header del bloque  : {}",hex_hdr);
    println!("el tx_count es {}",tx_count);
    println!("el version es {}",version);
    println!("el txin_count es {}",txin_count);
    println!("el txout_count es {}",txout_count);
    println!("el lock_time es {}",lock_time);
    println!("coin_hash: {}",coin_string);
*/
 
    /*// bloque 00000000000000127a638dfa7b517f1045217884cb986ab8f653b8be0ab37447
    let mut transactions: Vec<[u8; 32]> = Vec::new();
    let mut coinbase = string_to_bytes("129f32d171b2a0c4ad5fd21f7504ae483845d311214f79eb927db49dfb28b838").unwrap();
    coinbase.reverse();
    transactions.push(coinbase);
    let mut tx_1 = string_to_bytes("aefeb6fb10f2f6a63a3cd4f70f1b7f8b193881a10ae5832a595e938d1630f1b9").unwrap();
    tx_1.reverse();
    transactions.push(tx_1);
    let mut tx_2 = string_to_bytes("4b0d8fd869e252803909aed9642bc8af28ebd18f2c4045b9b41679eda0ff79dd").unwrap();
    tx_2.reverse();
    transactions.push(tx_2);
    let mut tx_3 = string_to_bytes("dbd558c896afe59a6dce2dc26bc32f4679b336ff0b1c0f2f8aaee846c5732333").unwrap();
    tx_3.reverse();
    transactions.push(tx_3);
    let mut tx_4 = string_to_bytes("88030de1d5f1b023893f8258df1796863756d99eef5c91a5528362f73497ac51").unwrap();
    tx_4.reverse();
    transactions.push(tx_4);
    println!("{:?}",transactions[0]);
    println!("{}",transactions.len());
    let mut  merkle_root = Block::recursive_generation_merkle_root(transactions);
    merkle_root.reverse();
    let hex_string = merkle_root.encode_hex::<String>();
    println!("{}",hex_string);
*/
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