use std::{
    error::Error,
    io,
    sync::{Arc, RwLock},
};

use gtk::glib;

use crate::{
    account::Account,
    blocks::utils_block::{make_merkle_proof, string_to_bytes},
    custom_errors::NodeCustomErrors,
    gtk::ui_events::{send_event_to_ui, UIEvent},
    node::Node,
};

#[derive(Debug, Clone)]

pub struct Wallet {
    pub node: Node,
    pub current_account_index: usize,
    pub accounts: Arc<RwLock<Vec<Account>>>,
}

impl Wallet {
    /// Crea la wallet. Inicializa el nodo con la referencia de las cuentas de la wallet
    pub fn new(node: Node) -> Result<Self, NodeCustomErrors> {
        let mut wallet = Wallet {
            node,
            current_account_index: 0,
            accounts: Arc::new(RwLock::new(Vec::new())),
        };
        wallet.node.set_accounts(wallet.accounts.clone())?;
        Ok(wallet)
    }

    /// Realiza una transacción y hace el broadcast.
    /// Recibe la cuenta que envía, la address receptora, monto y fee.
    /// Devuelve error en caso de que algo falle.
    pub fn make_transaction(
        &self,
        account_index: usize,
        address_receiver: &str,
        amount: i64,
        fee: i64,
    ) -> Result<(), Box<dyn Error>> {
        validate_transaction_data(self.accounts.clone(), account_index, amount, fee)?;
        let transaction_hash: [u8; 32] = self
            .accounts
            .write()
            .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?[account_index]
            .make_transaction(address_receiver, amount, fee)?;
        self.node.broadcast_tx(transaction_hash)?;
        Ok(())
    }

    /// Agrega una cuenta a la wallet.
    /// Devuelve error si las claves ingresadas son inválidas
    pub fn add_account(
        &mut self,
        ui_sender: &Option<glib::Sender<UIEvent>>,
        wif_private_key: String,
        address: String,
    ) -> Result<(), NodeCustomErrors> {
        let mut account = Account::new(wif_private_key, address).map_err(|err| {
            send_event_to_ui(ui_sender, UIEvent::AddAccountError(err.to_string()));
            NodeCustomErrors::UnmarshallingError(err.to_string())
        })?;
        self.load_data(&mut account)
            .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?;
        self.accounts
            .write()
            .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
            .push(account.clone());
        send_event_to_ui(ui_sender, UIEvent::AccountAddedSuccesfully(account));
        Ok(())
    }
    /// Funcion que se encarga de cargar los respectivos utxos asociados a la cuenta
    fn load_data(&self, account: &mut Account) -> Result<(), Box<dyn Error>> {
        let address = account.get_address().clone();
        let utxos_to_account = self.node.utxos_referenced_to_account(&address)?;
        account.load_utxos(utxos_to_account);
        Ok(())
    }

    /// Muestra el balance de las cuentas.
    pub fn show_accounts_balance(&self) -> Result<(), Box<dyn Error>> {
        if self
            .accounts
            .read()
            .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
            .is_empty()
        {
            println!("No hay cuentas en la wallet!");
        }
        for account in self
            .accounts
            .write()
            .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
            .iter()
        {
            println!(
                "Cuenta: {} - Balance: {:.8} tBTC",
                account.address,
                account.balance() as f64 / 1e8
            );
        }
        Ok(())
    }

    /// Muestra los indices que corresponden a cada cuenta
    pub fn show_indexes_of_accounts(&self) -> Result<(), Box<dyn Error>> {
        if self
            .accounts
            .read()
            .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
            .is_empty()
        {
            println!("No hay cuentas en la wallet. No es posible realizar una transaccion!");
            return Err(Box::new(std::io::Error::new(
                io::ErrorKind::Other,
                "No hay cuentas en la wallet. No es posible realizar una transaccion!",
            )));
        }
        println!("INDICES DE LAS CUENTAS");
        for (index, account) in self
            .accounts
            .read()
            .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
            .iter()
            .enumerate()
        {
            println!("{}: {}", index, account.address);
        }
        println!();
        Ok(())
    }

    /// Solicita al nodo la proof of inclusion de la transacción
    /// Recibe el hash de la transacción y del bloque en que se encuentra.
    /// Evalúa la POI y devuelve true o false
    pub fn tx_proof_of_inclusion(
        &self,
        block_hash_hex: String,
        tx_hash_hex: String,
    ) -> Result<bool, Box<dyn Error>> {
        let mut block_hash: [u8; 32] = string_to_bytes(&block_hash_hex)?;
        let mut tx_hash: [u8; 32] = string_to_bytes(&tx_hash_hex)?;
        block_hash.reverse();
        tx_hash.reverse();

        let poi = self.node.merkle_proof_of_inclusion(&block_hash, &tx_hash)?;

        let hashes = match poi {
            Some(value) => value,
            None => return Ok(false),
        };
        Ok(make_merkle_proof(&hashes, &tx_hash))
    }
}

fn validate_transaction_data(
    accounts: Arc<RwLock<Vec<Account>>>,
    account_index: usize,
    amount: i64,
    fee: i64,
) -> Result<(), Box<dyn Error>> {
    let accounts_len = accounts
        .read()
        .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?
        .len();
    if accounts_len < account_index {
        return Err(Box::new(std::io::Error::new(
            io::ErrorKind::Other,
            "El indice ingresado es incorrecto.",
        )));
    }

    if (amount + fee) <= 0 {
        return Err(Box::new(std::io::Error::new(
            io::ErrorKind::Other,
            "El monto a gastar debe ser mayor a cero.",
        )));
    }
    Ok(())
}
