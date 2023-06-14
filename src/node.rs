use std::{
    net::TcpStream,
    sync::{Arc, RwLock},
};

use crate::{
    blocks::{block::Block, block_header::BlockHeader},
    transactions::transaction::Transaction,
    utxo_tuple::UtxoTuple, account::Account, node_message_handler::{NodeMessageHandler, NodeMessageHandlerError}, logwriter::log_writer::LogSender,
};
#[derive(Debug, Clone)]

pub struct Node {
    pub connected_nodes: Arc<RwLock<Vec<TcpStream>>>,
    pub headers: Arc<RwLock<Vec<BlockHeader>>>,
    pub block_chain: Arc<RwLock<Vec<Block>>>,
    pub utxo_set: Vec<UtxoTuple>,
    pub accounts: Option<Arc<RwLock<Vec<Account>>>>,
    pub peers_handler: NodeMessageHandler,
}

impl Node {
    pub fn new(
        log_sender: LogSender,
        connected_nodes: Arc<RwLock<Vec<TcpStream>>>,
        headers: Arc<RwLock<Vec<BlockHeader>>>,
        block_chain: Arc<RwLock<Vec<Block>>>,
    ) -> Self {
        let utxo_set = generate_utxo_set(&block_chain);
        let peers_handler = NodeMessageHandler::new(log_sender, headers.clone(), block_chain.clone(), connected_nodes.clone()).unwrap();
        Node {
            connected_nodes,
            headers,
            block_chain,
            utxo_set,
            accounts: None,
            peers_handler,
        }
    }
    /// funcion para validar un bloque
    pub fn block_validation(block: Block) -> (bool, &'static str) {
        block.validate()
    }

    /// funcion que cargara las utxos asociadas a la respectiva cuenta
    pub fn utxos_referenced_to_account(&self, address: &str) -> Vec<UtxoTuple> {
        let mut account_utxo_set: Vec<UtxoTuple> = Vec::new();
        for utxo in &self.utxo_set {
            let aux_utxo = utxo.referenced_utxos(address);
            let utxo_to_push = match aux_utxo {
                Some(value) => value,
                None => continue,
            };
            account_utxo_set.push(utxo_to_push);
        }
        account_utxo_set
    } 
    
    pub fn shutdown_node(&self) -> Result<(), NodeMessageHandlerError> {
        self.peers_handler.finish()
    }
    
    /*
      pub fn make_transaction(
          &mut self,
          adress_receiver: &str,
          amount_to_spend: i64,
          account: User,
      ) -> Result<(), &'static str> {
          if account.has_balance(amount_to_spend) {
              return Err("no tenes saldo disponible para realizar la operacion");
          }
          account.make_transaction(adress_receiver, amount_to_spend);
          Ok(())
      }*/

    /// funcion que muestra si una transaccion se encuentra en un determinado bloque
    pub fn merkle_proof_of_inclusion(
        transaction: Transaction,
        block: Block,
        vector_hash: Vec<[u8; 32]>,
    ) -> bool {
        block.merkle_proof_of_inclusion(transaction.hash(), vector_hash)
    }
}

///Funcion que se encarga de generar la lista de utxos
fn generate_utxo_set(block_chain: &Arc<RwLock<Vec<Block>>>) -> Vec<UtxoTuple> {
    let mut list_of_utxos: Vec<UtxoTuple> = Vec::new();

    let block_chain_lock = block_chain.read().unwrap();

    for block in block_chain_lock.iter() {
        let utxos: Vec<UtxoTuple> = block.give_me_utxos();
        list_of_utxos.extend_from_slice(&utxos);
    }
    list_of_utxos
}
