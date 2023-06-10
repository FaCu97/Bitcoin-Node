use std::{
    error::Error,
    sync::{Arc, RwLock},
};

use crate::{
    account::Account,
    blocks::{block::Block, block_header::BlockHeader},
    transactions::{transaction::Transaction, tx_out::TxOut},
};

pub struct Node {
    pub headers: Arc<RwLock<Vec<BlockHeader>>>,
    pub block_chain: Arc<RwLock<Vec<Block>>>,
    pub utxo_set: Vec<TxOut>,
    accounts: Vec<Account>,
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
            accounts: vec![],
        }
    }
    /// funcion para validar un bloque
    pub fn block_validation(block: Block) -> (bool, &'static str) {
        block.validate()
    }

    /// funcion que mostrara la cantidad de satoshis en nuestra cuenta
    pub fn account_balance(&self, adress: String) -> i64 {
        let mut account_balance: i64 = 0;
        for utxo in &self.utxo_set {
            match utxo.get_adress() {
                Ok(value) => {
                    if value == adress {
                        account_balance += utxo.value()
                    }
                }
                Err(_) => {
                    continue;
                }
            }
        }
        account_balance
    }
    pub fn make_transaction(
        &mut self,
        adress_receiver: &str,
        amount_to_spend: i64,
    ) -> Result<(), &'static str> {
        Ok(())
    }

    /// funcion que muestra si una transaccion se encuentra en un determinado bloque
    pub fn merkle_proof_of_inclusion(
        transaction: Transaction,
        block: Block,
        vector_hash: Vec<[u8; 32]>,
    ) -> bool {
        block.merkle_proof_of_inclusion(transaction.hash(), vector_hash)
    }

    pub fn add_account(
        &mut self,
        wif_private_key: String,
        address: String,
    ) -> Result<(), Box<dyn Error>> {
        self.accounts.push(Account::new(wif_private_key, address)?);
        Ok(())
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

#[cfg(test)]
mod test {
    use crate::{account::Account, node::Node};
    use std::{
        error::Error,
        sync::{Arc, RwLock},
    };

    #[test]
    fn test_una_address_se_registra_correctamente() -> Result<(), Box<dyn Error>> {
        let address: String = String::from("mnEvYsxexfDEkCx2YLEfzhjrwKKcyAhMqV");
        let private_key: String =
            String::from("cMoBjaYS6EraKLNqrNN8DvN93Nnt6pJNfWkYM8pUufYQB5EVZ7SR");
        let blocks = Arc::new(RwLock::new(Vec::new()));
        let headers = Arc::new(RwLock::new(Vec::new()));

        let mut node = Node::new(headers, blocks);
        let account_addecd_result = node.add_account(private_key, address);

        assert!(account_addecd_result.is_ok());
        Ok(())
    }
}
