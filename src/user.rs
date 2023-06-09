use std::error::Error;
use std::io;

use bitcoin_hashes::{ripemd160, Hash};
use k256::sha2::Digest;
use k256::sha2::Sha256;
use secp256k1::SecretKey;

use crate::node::Node;
const UNCOMPRESSED_WIF_LEN: usize = 51;

pub struct User {
    private_key: String,
    address: String,
}

impl User {
    /// Recibe La address en formato comprimido
    /// Y la WIF private key, ya sea en formato comprimido o no comprimido
    pub fn login_user(wif_private_key: String, address: String) -> Result<User, Box<dyn Error>> {
        let raw_private_key = Self::decode_wif_private_key(wif_private_key.as_str())?;

        Self::validate_address_private_key(&raw_private_key, &address)?;
        Ok(User {
            private_key: wif_private_key,
            address,
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
        //Err("Falló la decodificación del wif private key en base58."),

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

    pub fn get_account_balance(&self, node: &Node) -> i64 {
        node.account_balance(self.address.clone())
    }
}

#[cfg(test)]

mod test {
    use std::error::Error;

    use hex;

    use crate::user::User;

    fn string_to_bytes(input: &str) -> Result<[u8; 32], hex::FromHexError> {
        let bytes = hex::decode(input)?;
        let mut result = [0; 32];
        result.copy_from_slice(&bytes[..32]);
        Ok(result)
    }

    #[test]
    fn test_se_genera_correctamente_el_usuario_con_wif_comprimida() {
        let address_expected: String = String::from("mnEvYsxexfDEkCx2YLEfzhjrwKKcyAhMqV");
        let private_key: String =
            String::from("cMoBjaYS6EraKLNqrNN8DvN93Nnt6pJNfWkYM8pUufYQB5EVZ7SR");
        let user = User::login_user(private_key, address_expected);
        assert!(user.is_ok());
    }

    #[test]
    fn test_se_genera_correctamente_el_usuario_con_wif_no_comprimida() {
        let address_expected: String = String::from("mnEvYsxexfDEkCx2YLEfzhjrwKKcyAhMqV");
        let private_key: String =
            String::from("91dkDNCCaMp2f91sVQRGgdZRw1QY4aptaeZ4vxEvuG5PvZ9hftJ");
        let user = User::login_user(private_key, address_expected);
        assert!(user.is_ok());
    }

    #[test]
    fn test_no_se_puede_generar_el_usuario_con_wif_incorrecta() {
        let address_expected: String = String::from("mnEvYsxexfDEkCx2YLEfzhjrwKKcyAhMqV");
        let private_key: String =
            String::from("K1dkDNCCaMp2f91sVQRGgdZRw1QY4aptaeZ4vxEvuG5PvZ9hftJ");
        let user = User::login_user(private_key, address_expected);
        assert!(user.is_err());
    }

    #[test]
    fn test_decoding_wif_compressed_genera_correctamente_el_private_key(
    ) -> Result<(), Box<dyn Error>> {
        // WIF COMPRESSED
        let wif = "cMoBjaYS6EraKLNqrNN8DvN93Nnt6pJNfWkYM8pUufYQB5EVZ7SR";
        // PRIVATE KEY FROM HEX FORMAT
        let expected_private_key_bytes =
            string_to_bytes("066C2068A5B9D650698828A8E39F94A784E2DDD25C0236AB7F1A014D4F9B4B49")
                .unwrap();

        let private_key = User::decode_wif_private_key(wif)?;

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
            string_to_bytes("066C2068A5B9D650698828A8E39F94A784E2DDD25C0236AB7F1A014D4F9B4B49")
                .unwrap();

        let private_key = User::decode_wif_private_key(wif)?;
        assert_eq!(private_key.to_vec(), expected_private_key_bytes);
        Ok(())
    }
}
