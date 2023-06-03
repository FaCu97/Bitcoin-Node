use bitcoin_hashes::{ripemd160, Hash};
use k256::sha2::Digest;
use k256::sha2::Sha256;
use secp256k1::SecretKey;

use crate::node::Node;

pub struct User {
    private_key: String,
    adress: String,
}

impl User {
    pub fn login_user(private_key: String, adress: String) -> Result<User, &'static str> {
        let raw_private_key = Self::decode_wif_private_key(private_key.as_str());
        let priv_key: [u8; 32];
        match raw_private_key {
            Some(number) => priv_key = number,
            None => return Err("fallo la decodificacion de la clave"),
        }
        let validate_adress = Self::generate_adress(&priv_key);
        if validate_adress == adress {
            return Ok(User {
                private_key,
                adress,
            });
        }
        Err("los datos ingresados no corresponden a un usuario valido")
    }

    fn generate_adress(private_key: &[u8]) -> String {
        // se aplica el algoritmo de ECDSA a la clave privada , luego
        // a la clave publica
        let secp: secp256k1::Secp256k1<secp256k1::All> = secp256k1::Secp256k1::new();
        let key: SecretKey = SecretKey::from_slice(private_key).unwrap();
        let public_key: secp256k1::PublicKey = secp256k1::PublicKey::from_secret_key(&secp, &key);
        //  se aplica RIPEMD160(SHA256(ECDSA(public_key)))
        let public_key_hexa = public_key.serialize_uncompressed();
        // let pk_hex: String = public_key_hexa.encode_hex::<String>();

        let sha256_hash = Sha256::digest(&public_key_hexa);
        let ripemd160_hash = *ripemd160::Hash::hash(&sha256_hash).as_byte_array();

        // Añadir el byte de versión (0x00) al comienzo del hash RIPEMD-160
        let mut extended_hash = vec![0x6f];
        extended_hash.extend_from_slice(&ripemd160_hash);

        // Calcular el checksum (doble hash SHA-256) del hash extendido
        let checksum = Sha256::digest(&Sha256::digest(&extended_hash));

        // Añadir los primeros 4 bytes del checksum al final del hash extendido
        extended_hash.extend_from_slice(&checksum[..4]);

        // Codificar el hash extendido en Base58
        let encoded: bs58::encode::EncodeBuilder<&Vec<u8>> = bs58::encode(&extended_hash);
        encoded.into_string()
    }

    pub fn decode_wif_private_key(wif_private_key: &str) -> Option<[u8; 32]> {
        // Decodificar la clave privada en formato WIF
        let decoded = bs58::decode(wif_private_key).into_vec().ok()?;
        let mut vector = vec![];
        vector.extend_from_slice(&decoded[1..&decoded.len() - 5]);
        // Obtener la clave privada de 32 bytes
        let mut private_key_bytes = [0u8; 32];
        private_key_bytes.copy_from_slice(&vector);
        Some(private_key_bytes)
    }

    pub fn get_account_balance(&self, node: &Node) -> i64 {
        node.account_balance(self.adress.clone())
    }
}

#[cfg(test)]

mod test {
    use hex;

    use crate::user::User;

    fn string_to_bytes(input: &str) -> Result<[u8; 32], hex::FromHexError> {
        let bytes = hex::decode(input)?;
        let mut result = [0; 32];
        result.copy_from_slice(&bytes[..32]);
        Ok(result)
    }

    #[test]
    fn test_se_genera_correctamente_el_usuario() {
        let adress_expected: String = String::from("msknbbUREqQw9worGo17T8BwsHSEVScx5C");
        let private_key: String =
            String::from("cMoBjaYS6EraKLNqrNN8DvN93Nnt6pJNfWkYM8pUufYQB5EVZ7SR");
        let user: Result<User, &str> = User::login_user(private_key, adress_expected);
        assert!(user.is_ok());
    }

    #[test]
    fn test_decoding_wif_genera_correctamente_el_adress() {
        // WIF COMPRIMIDA
        let wif = "cMoBjaYS6EraKLNqrNN8DvN93Nnt6pJNfWkYM8pUufYQB5EVZ7SR";
        // PUBLIC KEY SIN
        let expected_pk =
            string_to_bytes("066C2068A5B9D650698828A8E39F94A784E2DDD25C0236AB7F1A014D4F9B4B49")
                .unwrap();

        let pk = match User::decode_wif_private_key(wif) {
            Some(private_key) => private_key,
            None => [0; 32],
        };
        assert_eq!(pk.to_vec(), expected_pk);
    }
}
