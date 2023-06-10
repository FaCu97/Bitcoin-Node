use std::sync::{Arc, RwLock};

use crate::{
    blocks::{block::Block, block_header::BlockHeader},
    transactions::{pubkey, transaction::Transaction, tx_out::TxOut},
    user::User,
};

pub struct Node {
    pub headers: Arc<RwLock<Vec<BlockHeader>>>,
    pub block_chain: Arc<RwLock<Vec<Block>>>,
    pub utxo_set: Vec<TxOut>,
}

impl Node {
    pub fn new(
        headers: Arc<RwLock<Vec<BlockHeader>>>,
        block_chain: Arc<RwLock<Vec<Block>>>,
    ) -> Self {
        let utxo_set = generate_utxo_set(&block_chain);
        Node {
            headers,
            block_chain,
            utxo_set,
        }
    }
    /// funcion para validar un bloque
    pub fn block_validation(block: Block) -> (bool, &'static str) {
        block.validate()
    }

    /// funcion que cargara las utxos asociadas a la respectiva cuenta
    pub fn utxos_referenced_to_account(&self, adress: String) -> Vec<TxOut> {
        let mut utxo_set: Vec<TxOut> = Vec::new();
        for utxo in &self.utxo_set {
            match utxo.get_adress() {
                Ok(value) => {
                    if value == adress {
                        utxo_set.push(utxo.clone());
                    }
                }
                Err(_) => {
                    continue;
                }
            }
        }
        utxo_set
    } /*
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
fn generate_utxo_set(block_chain: &Arc<RwLock<Vec<Block>>>) -> Vec<TxOut> {
    let mut list_of_utxos = Vec::new();

    {
        let block_chain_lock = block_chain.read().unwrap();

        for block in block_chain_lock.iter() {
            let utxos = block.give_me_utxos();
            list_of_utxos.extend_from_slice(&utxos);
        }
    }

    list_of_utxos
}
