use std::{error::Error, sync::{RwLock, Arc}};

use crate::{account::Account, node::Node};
#[derive(Debug, Clone)]

pub struct Wallet {
    pub node: Node,
    pub current_account_index: usize,
    pub accounts: Arc<RwLock<Vec<Account>>>,
}

impl Wallet {
    pub fn new(node: Node, accounts: Vec<Account>) -> Wallet {
        let pointer_to_accounts = Arc::new(RwLock::new(accounts));
        let mut wallet = Wallet {
            node,
            current_account_index: 0,
            accounts: pointer_to_accounts.clone(),
        };
        wallet.node.set_accounts(pointer_to_accounts);
        wallet
    }

    pub fn make_transaction(
        &self,
        account: &Account,
        address_receiver: &str,
        amount: i64,
    ) -> Result<(), Box<dyn Error>> {
        let transaction = account.make_transaction(address_receiver, amount)?;
        // self.node.broadcast_transaction()?;
        Ok(())
    }

    /// Agrega una cuenta a la wallet.
    /// Devuelve error si las claves ingresadas son inválidas
    pub fn add_account(
        &mut self,
        wif_private_key: String,
        address: String,
    ) -> Result<(), Box<dyn Error>> {
        let mut account = Account::new(wif_private_key, address)?;
        self.load_data(&mut account);
        self.accounts.write().unwrap().push(account);
        Ok(())
    }
    /// Funcion que se encarga de cargar los respectivos utxos asociados a la cuenta
    fn load_data(&self, account: &mut Account) {
        let address = account.get_address().clone();
        let utxos_to_account = self.node.utxos_referenced_to_account(&address);
        account.load_utxos(utxos_to_account);
    }
}


/* 
#[cfg(test)]
mod test {
    use crate::{account::Account, node::Node, wallet::Wallet};
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

        let node = Node::new(Arc::new(RwLock::new(vec![])), headers, blocks);
        let mut wallet = Wallet::new(node);
        let account_addecd_result = wallet.add_account(private_key, address);

        assert!(account_addecd_result.is_ok());
        Ok(())
    }
}
*/
