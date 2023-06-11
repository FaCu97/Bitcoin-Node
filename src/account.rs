use std::error::Error;
use std::io;
use std::sync::Arc;
use std::sync::RwLock;

use bitcoin_hashes::{ripemd160, Hash};
use k256::sha2::Digest;
use k256::sha2::Sha256;
use secp256k1::SecretKey;

use crate::transactions::transaction::Transaction;
use crate::utxo_tuple::UtxoTuple;
const UNCOMPRESSED_WIF_LEN: usize = 51;
#[derive(Debug, Clone)]

/// Guarda la address comprimida y la private key (comprimida o no)
pub struct Account {
    pub private_key: String,
    pub address: String,
    pub utxo_set: Vec<UtxoTuple>,
    pub pending_transactions: Arc<RwLock<Vec<Transaction>>>
}

impl Account {
    /// Recibe la address en formato comprimido
    /// Y la WIF private key, ya sea en formato comprimido o no comprimido
    pub fn new(wif_private_key: String, address: String) -> Result<Account, Box<dyn Error>> {
        let raw_private_key = Self::decode_wif_private_key(wif_private_key.as_str())?;

        Self::validate_address_private_key(&raw_private_key, &address)?;
        Ok(Account {
            private_key: wif_private_key,
            address,
            utxo_set: Vec::new(),
            pending_transactions: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Recibe una private key en bytes y una address comprimida.
    /// Devuelve true o false dependiendo si se corresponden entre si o no.
    fn validate_address_private_key(
        private_key: &[u8],
        address: &String,
    ) -> Result<(), Box<dyn Error>> {
        if !Self::generate_address(private_key)?.eq(address) {
            return Err(Box::new(std::io::Error::new(
                io::ErrorKind::Other,
                "La private key ingresada no se corresponde con la address",
            )));
        }
        Ok(())
    }

    /// Recibe la private key en bytes.
    /// Devuelve la address comprimida
    fn generate_address(private_key: &[u8]) -> Result<String, Box<dyn Error>> {
        // se aplica el algoritmo de ECDSA a la clave privada , luego
        // a la clave publica
        let secp: secp256k1::Secp256k1<secp256k1::All> = secp256k1::Secp256k1::new();
        let key = SecretKey::from_slice(private_key)?;
        let public_key: secp256k1::PublicKey = secp256k1::PublicKey::from_secret_key(&secp, &key);
        let public_key_bytes_compressed = public_key.serialize();

        //  se aplica RIPEMD160(SHA256(ECDSA(public_key)))
        let sha256_hash = Sha256::digest(public_key_bytes_compressed);
        let ripemd160_hash = *ripemd160::Hash::hash(&sha256_hash).as_byte_array();

        // Añadir el byte de versión (0x00) al comienzo del hash RIPEMD-160
        let mut extended_hash = vec![0x6f];
        extended_hash.extend_from_slice(&ripemd160_hash);

        // Calcular el checksum (doble hash SHA-256) del hash extendido
        let checksum = Sha256::digest(Sha256::digest(&extended_hash));

        // Añadir los primeros 4 bytes del checksum al final del hash extendido
        extended_hash.extend_from_slice(&checksum[..4]);

        // Codificar el hash extendido en Base58
        let encoded: bs58::encode::EncodeBuilder<&Vec<u8>> = bs58::encode(&extended_hash);
        Ok(encoded.into_string())
    }

    /// Recibe la WIF private key, ya sea en formato comprimido o no comprimido.
    /// Devuelve la private key en bytes
    pub fn decode_wif_private_key(wif_private_key: &str) -> Result<[u8; 32], Box<dyn Error>> {
        // Decodificar la clave privada en formato WIF
        let decoded_result = bs58::decode(wif_private_key).into_vec();
        let decoded = match decoded_result {
            Ok(decoded) => decoded,
            Err(err) => return Err(Box::new(std::io::Error::new(io::ErrorKind::Other, err))),
        };

        let mut vector = vec![];
        if wif_private_key.len() == UNCOMPRESSED_WIF_LEN {
            vector.extend_from_slice(&decoded[1..&decoded.len() - 4]);
        } else {
            vector.extend_from_slice(&decoded[1..&decoded.len() - 5]);
        }

        if vector.len() != 32 {
            return Err(Box::new(std::io::Error::new(
                io::ErrorKind::Other,
                "No se pudo decodificar la WIF private key.",
            )));
        }

        // Obtener la clave privada de 32 bytes
        let mut private_key_bytes = [0u8; 32];
        private_key_bytes.copy_from_slice(&vector);

        Ok(private_key_bytes)
    }
    /*
        pub fn get_account_balance(&self, node: &Node) -> i64 {
            node.account_balance(self.address.clone())
        }
    */
    /// Devuelve la clave publica comprimida (33 bytes) a partir de la privada
    pub fn get_pubkey_compressed(&self) -> Result<[u8; 33], Box<dyn Error>> {
        let private_key = Self::decode_wif_private_key(self.private_key.as_str())?;
        let secp: secp256k1::Secp256k1<secp256k1::All> = secp256k1::Secp256k1::new();
        let key: SecretKey = SecretKey::from_slice(&private_key).unwrap();
        let public_key: secp256k1::PublicKey = secp256k1::PublicKey::from_secret_key(&secp, &key);
        Ok(public_key.serialize())
    }
    pub fn get_private_key(&self) -> Result<[u8; 32], Box<dyn Error>> {
        Self::decode_wif_private_key(self.private_key.as_str())
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

    pub fn make_transaction(
        &self,
        address_receiver: &str,
        amount: i64,
    ) -> Result<(), Box<dyn Error>> {
        if !self.has_balance(amount) {
            return Err(Box::new(std::io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "El balance de la cuenta {} tiene menos de {} satoshis",
                    self.address, amount,
                ),
            )));
        }
        //let transaction: Transaction::generate_transaction_to(address: &str, amount: i64)?;
        // letTransaction::new(...)
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use std::{error::Error, sync::{RwLock, Arc}};

    use hex;

    use crate::account::Account;

    fn string_to_32_bytes(input: &str) -> Result<[u8; 32], hex::FromHexError> {
        let bytes = hex::decode(input)?;
        let mut result = [0; 32];
        result.copy_from_slice(&bytes[..32]);
        Ok(result)
    }
    fn string_to_33_bytes(input: &str) -> Result<[u8; 33], hex::FromHexError> {
        let bytes = hex::decode(input)?;
        let mut result = [0; 33];
        result.copy_from_slice(&bytes[..33]);
        Ok(result)
    }

    #[test]
    fn test_se_genera_correctamente_el_usuario_con_wif_comprimida() {
        let address_expected: String = String::from("mnEvYsxexfDEkCx2YLEfzhjrwKKcyAhMqV");
        let private_key: String =
            String::from("cMoBjaYS6EraKLNqrNN8DvN93Nnt6pJNfWkYM8pUufYQB5EVZ7SR");
        let account_result = Account::new(private_key, address_expected);
        assert!(account_result.is_ok());
    }

    #[test]
    fn test_se_genera_correctamente_el_usuario_con_wif_no_comprimida() {
        let address_expected: String = String::from("mnEvYsxexfDEkCx2YLEfzhjrwKKcyAhMqV");
        let private_key: String =
            String::from("91dkDNCCaMp2f91sVQRGgdZRw1QY4aptaeZ4vxEvuG5PvZ9hftJ");
        let account_result = Account::new(private_key, address_expected);
        assert!(account_result.is_ok());
    }

    #[test]
    fn test_no_se_puede_generar_el_usuario_con_wif_incorrecta() {
        let address_expected: String = String::from("mnEvYsxexfDEkCx2YLEfzhjrwKKcyAhMqV");
        let private_key: String =
            String::from("K1dkDNCCaMp2f91sVQRGgdZRw1QY4aptaeZ4vxEvuG5PvZ9hftJ");
        let account_result = Account::new(private_key, address_expected);
        assert!(account_result.is_err());
    }

    #[test]
    fn test_decoding_wif_compressed_genera_correctamente_el_private_key(
    ) -> Result<(), Box<dyn Error>> {
        // WIF COMPRESSED
        let wif = "cMoBjaYS6EraKLNqrNN8DvN93Nnt6pJNfWkYM8pUufYQB5EVZ7SR";
        // PRIVATE KEY FROM HEX FORMAT
        let expected_private_key_bytes =
            string_to_32_bytes("066C2068A5B9D650698828A8E39F94A784E2DDD25C0236AB7F1A014D4F9B4B49")
                .unwrap();

        let private_key = Account::decode_wif_private_key(wif)?;

        assert_eq!(private_key.to_vec(), expected_private_key_bytes);
        Ok(())
    }

    #[test]
    fn test_decoding_wif_uncompressed_genera_correctamente_el_private_key(
    ) -> Result<(), Box<dyn Error>> {
        // WIF UNCOMPRESSED
        let wif = "91dkDNCCaMp2f91sVQRGgdZRw1QY4aptaeZ4vxEvuG5PvZ9hftJ";
        // PRIVATE KEY FROM HEX FORMAT
        let expected_private_key_bytes =
            string_to_32_bytes("066C2068A5B9D650698828A8E39F94A784E2DDD25C0236AB7F1A014D4F9B4B49")
                .unwrap();

        let private_key = Account::decode_wif_private_key(wif)?;
        assert_eq!(private_key.to_vec(), expected_private_key_bytes);
        Ok(())
    }
    #[test]
    fn test_usuario_devuelve_clave_publica_comprimida_esperada() {
        let address = String::from("mpzx6iZ1WX8hLSeDRKdkLatXXPN1GDWVaF");
        let private_key = String::from("cQojsQ5fSonENC5EnrzzTAWSGX8PB4TBh6GunBxcCdGMJJiLULwZ");
        let user = Account {
            private_key,
            address,
            utxo_set: Vec::new(),
            pending_transactions: Arc::new(RwLock::new(Vec::new()))
        };
        let expected_pubkey = string_to_33_bytes(
            "0345EC0AA86BAF64ED626EE86B4A76C12A92D5F6DD1C1D6E4658E26666153DAFA6",
        )
        .unwrap();
        assert_eq!(user.get_pubkey_compressed().unwrap(), expected_pubkey);
    }
}
