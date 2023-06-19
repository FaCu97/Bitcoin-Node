use std::error::Error;
use std::io;
use std::sync::Arc;
use std::sync::RwLock;

use crate::address_decoder;
use crate::transactions::transaction::Transaction;
use crate::utxo_tuple::UtxoTuple;
#[derive(Debug, Clone)]

/// Guarda la address comprimida y la private key (comprimida o no)
pub struct Account {
    pub private_key: String,
    pub address: String,
    pub utxo_set: Vec<UtxoTuple>,
    pub pending_transactions: Arc<RwLock<Vec<Transaction>>>,
    pub confirmed_transactions: Arc<RwLock<Vec<Transaction>>>,
}

impl Account {
    /// Recibe la address en formato comprimido
    /// Y la WIF private key, ya sea en formato comprimido o no comprimido
    pub fn new(wif_private_key: String, address: String) -> Result<Account, Box<dyn Error>> {
        let raw_private_key = address_decoder::decode_wif_private_key(wif_private_key.as_str())?;

        address_decoder::validate_address_private_key(&raw_private_key, &address)?;
        Ok(Account {
            private_key: wif_private_key,
            address,
            utxo_set: Vec::new(),
            pending_transactions: Arc::new(RwLock::new(Vec::new())),
            confirmed_transactions: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Devuelve la clave publica comprimida (33 bytes) a partir de la privada
    pub fn get_pubkey_compressed(&self) -> Result<[u8; 33], Box<dyn Error>> {
        address_decoder::get_pubkey_compressed(&self.private_key)
    }
    pub fn get_private_key(&self) -> Result<[u8; 32], Box<dyn Error>> {
        address_decoder::decode_wif_private_key(self.private_key.as_str())
    }
    pub fn get_address(&self) -> &String {
        &self.address
    }
    pub fn load_utxos(&mut self, utxos: Vec<UtxoTuple>) {
        self.utxo_set.extend_from_slice(&utxos);
    }
    pub fn has_balance(&self, value: i64) -> bool {
        let mut balance: i64 = 0;
        for utxo in &self.utxo_set {
            balance += utxo.balance();
        }
        balance > value
    }
    /// Devuelve un vector con las utxos a ser gastadas en una transaccion nueva
    fn get_utxos_for_amount(&mut self, value: i64) -> Vec<UtxoTuple> {
        let mut utxos_to_spend = Vec::new();
        let mut partial_amount: i64 = 0;
        let mut position: usize = 0;
        let lenght: usize = self.utxo_set.len();
        while position < lenght {
            if (partial_amount + self.utxo_set[position].balance()) < value {
                partial_amount += self.utxo_set[position].balance();
                utxos_to_spend.push(self.utxo_set[position].clone());
                // No corresponde removerlas mientras la tx no está confirmada
                // self.remove_utxo(position);
            } else {
                utxos_to_spend
                    .push(self.utxo_set[position].utxos_to_spend(value, &mut partial_amount));
                break;
            }
            position += 1;
        }
        utxos_to_spend
    }

    fn remove_utxo(&mut self, position: usize) {
        self.utxo_set.remove(position);
    }

    fn add_transaction(&self, transaction: Transaction) {
        let mut aux = self.pending_transactions.write().unwrap();
        aux.push(transaction);
    }
    /// Realiza la transaccion con el monto recibido , devuelve el hash de dicha transaccion
    /// para que el nodo envie dicho hash a lo restantes nodos de la red
    pub fn make_transaction(
        &mut self,
        address_receiver: &str,
        amount: i64,
    ) -> Result<[u8; 32], Box<dyn Error>> {
        if !self.has_balance(amount) {
            return Err(Box::new(std::io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "El balance de la cuenta {} tiene menos de {} satoshis",
                    self.address, amount,
                ),
            )));
        }
        // sabemos que tenemos monto para realizar la transaccion , ahora debemos obtener las utxos
        // que utilizaremos para gastar
        let utxos_to_spend: Vec<UtxoTuple> = self.get_utxos_for_amount(amount);
        let mut unsigned_transaction =
            Transaction::generate_unsigned_transaction(address_receiver, amount, &utxos_to_spend)?;
        unsigned_transaction.sign(&self, &utxos_to_spend)?;
        let mut bytes = Vec::new();
        unsigned_transaction.marshalling(&mut bytes);
        println!(
            "RAW TRANSACTION: {:?}",
            bytes_to_hex_string(&bytes.to_vec())
        );
        self.add_transaction(unsigned_transaction.clone());
        Ok(unsigned_transaction.hash())
    }
}
pub fn bytes_to_hex_string(bytes: &[u8]) -> String {
    let hex_chars: Vec<String> = bytes.iter().map(|byte| format!("{:02x}", byte)).collect();

    hex_chars.join("")
}

#[cfg(test)]
mod test {

    use std::{
        error::Error,
        sync::{Arc, RwLock},
    };

    use hex;

    use crate::account::Account;

    fn string_to_33_bytes(input: &str) -> Result<[u8; 33], hex::FromHexError> {
        let bytes = hex::decode(input)?;
        let mut result = [0; 33];
        result.copy_from_slice(&bytes[..33]);
        Ok(result)
    }

    #[test]
    fn test_se_genera_correctamente_la_cuenta_con_wif_comprimida() {
        let address_expected: String = String::from("mnEvYsxexfDEkCx2YLEfzhjrwKKcyAhMqV");
        let private_key: String =
            String::from("cMoBjaYS6EraKLNqrNN8DvN93Nnt6pJNfWkYM8pUufYQB5EVZ7SR");
        let account_result = Account::new(private_key, address_expected);
        assert!(account_result.is_ok());
    }

    #[test]
    fn test_se_genera_correctamente_la_cuenta_con_wif_no_comprimida() {
        let address_expected: String = String::from("mnEvYsxexfDEkCx2YLEfzhjrwKKcyAhMqV");
        let private_key: String =
            String::from("91dkDNCCaMp2f91sVQRGgdZRw1QY4aptaeZ4vxEvuG5PvZ9hftJ");
        let account_result = Account::new(private_key, address_expected);
        assert!(account_result.is_ok());
    }

    #[test]
    fn test_no_se_puede_generar_la_cuenta_con_wif_incorrecta() {
        let address_expected: String = String::from("mnEvYsxexfDEkCx2YLEfzhjrwKKcyAhMqV");
        let private_key: String =
            String::from("K1dkDNCCaMp2f91sVQRGgdZRw1QY4aptaeZ4vxEvuG5PvZ9hftJ");
        let account_result = Account::new(private_key, address_expected);
        assert!(account_result.is_err());
    }

    #[test]
    fn test_usuario_devuelve_clave_publica_comprimida_esperada() -> Result<(), Box<dyn Error>> {
        let address = String::from("mpzx6iZ1WX8hLSeDRKdkLatXXPN1GDWVaF");
        let private_key = String::from("cQojsQ5fSonENC5EnrzzTAWSGX8PB4TBh6GunBxcCdGMJJiLULwZ");
        let user = Account {
            private_key,
            address,
            utxo_set: Vec::new(),
            pending_transactions: Arc::new(RwLock::new(Vec::new())),
            confirmed_transactions: Arc::new(RwLock::new(Vec::new())),
        };
        let expected_pubkey = string_to_33_bytes(
            "0345EC0AA86BAF64ED626EE86B4A76C12A92D5F6DD1C1D6E4658E26666153DAFA6",
        )
        .unwrap();
        assert_eq!(user.get_pubkey_compressed()?, expected_pubkey);
        Ok(())
    }
}
