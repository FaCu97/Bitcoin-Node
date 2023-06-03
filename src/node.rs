use std::sync::{Arc, RwLock};

use crate::{
    blocks::{block::Block, block_header::BlockHeader},
    transactions::{transaction::Transaction, tx_out::TxOut},
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

    /// funcion que mostrara la cantidad de satoshis en nuestra cuenta
    pub fn account_balance(&self) -> i64 {
        let mut account_balance: i64 = 0;
        for utxo in &self.utxo_set {
            account_balance += utxo.value()
        }
        account_balance
    }
    pub fn make_transaction(&mut self, _adress: [u8; 32], amount_to_spend: i64) -> bool {
        let mut position_utxo: usize = 0;
        let mut can_spend: bool = false;
        while position_utxo < self.utxo_set.len() && !can_spend {
            if self.utxo_set[position_utxo].value() > amount_to_spend {
                can_spend = true
            }
            position_utxo += 1;
        }
        if can_spend {
            let _utxo_to_spend: &TxOut = &self.utxo_set[position_utxo - 1];
            self.utxo_set.remove(position_utxo - 1);
            return true;
        }
        false
    }

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
