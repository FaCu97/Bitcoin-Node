use crate::{account::Account, node::Node};

pub struct Wallet {
    node: Node,
    // current_account : User,
    accounts: Vec<Account>,
}

impl Wallet {

    pub fn new(node: Node){
        Wallet{
            node,
            accounts: vec![],
        }
    }

    pub fn make_transaction(account: Account, adress: &str, value: i64) -> Result<(), &'static str> {
        if !account.has_balance(value) {
            return Err(
                "El balance de la cuenta {} tiene menos de {} satoshis",
                account.address,
                value,
            );
        }
        self.node.make_transaction(account.get_adress())
    }
    pub fn add_account(
        &mut self,
        wif_private_key: String,
        address: String,
    ) -> Result<(), Box<dyn Error>> {
        let account = Account::new(wif_private_key, address)?;
        // ver utxos
        self.accounts.push(account);
        Ok(())
    }

    pub fn load_data(&self,account:&Account){
        let utxos_to_account = self.node
        
    }
}


#[cfg(test)]
mod test {
    use crate::{account::Account, node::Node};
    use std::{
        error::Error,
        sync::{Arc, RwLock},
    };


/* 
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
*/