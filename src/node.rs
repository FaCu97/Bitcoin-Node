use crate::{
    blocks::{block::Block, block_header::BlockHeader},
    transactions::{transaction::Transaction, tx_out::TxOut},
};

pub struct Node {
    pub headers: Vec<BlockHeader>,
    pub block_chain: Vec<Block>,
    pub utxo_set: Vec<TxOut>,
}

impl Node {
    // funcion para validar un bloque
    pub fn block_validation(block: Block) -> (bool, &'static str) {
        block.validate()
    }

    // funcion que mostrara la cantidad de satoshis en nuestra cuenta
    pub fn account_balance(&self) -> i64 {
        let mut account_balance: i64 = 0;
        for utxo in &self.utxo_set {
            account_balance += utxo.value()
        }
        account_balance
    }
    pub fn make_transaction(&mut self, _adress: [u8; 32], amount_to_spend: i64) -> bool {
        let mut position_utxo: usize = 0;
        for utxo in &self.utxo_set {
            if utxo.value() > amount_to_spend {
                break;
            }
            position_utxo += 1;
        }
        let _utxo_to_spend: &TxOut = &self.utxo_set[position_utxo];
        self.utxo_set.remove(position_utxo);
        false
    }

    // funcion que muestra si una transaccion se encuentra en un determinado bloque
    pub fn merkle_proof_of_inclusion(
        transaction: Transaction,
        block: Block,
        vector_hash: Vec<[u8; 32]>,
    ) -> bool {
        block.merkle_proof_of_inclusion(&transaction, vector_hash)
    }
}
