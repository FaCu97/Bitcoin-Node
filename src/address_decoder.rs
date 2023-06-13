use std::error::Error;
use std::io;

use k256::sha2::Digest;
use k256::sha2::Sha256;

/// Recibe la address comprimida
/// Devuelve el PubkeyHash
/// Si la address es invalida, devuelve error
pub fn get_pubkey_hash_from_address(address: &str) -> Result<[u8; 20], Box<dyn Error>> {
    //se decodifican de &str a bytes , desde el formate base58  a bytes
    let address_decoded_bytes = bs58::decode(address).into_vec()?;
    validate_address(&address_decoded_bytes)?;
    let lenght_bytes = address_decoded_bytes.len();
    let mut pubkey_hash: [u8; 20] = [0; 20];

    // el pubkey hash es el que compone la address
    // le saco el byte de la red y el checksum del final
    pubkey_hash.copy_from_slice(&address_decoded_bytes[1..(lenght_bytes - 4)]);

    Ok(pubkey_hash)
}

/// Recibe una bitcoin address decodificada.
/// Revisa el checksum y devuelve error si es inválida.
fn validate_address(address_decoded_bytes: &Vec<u8>) -> Result<(), Box<dyn Error>> {
    // validacion checksum: evita errores de tipeo en la address
    // Calcular el checksum (doble hash SHA-256) del hash extendido
    let lenght_bytes = address_decoded_bytes.len();
    let checksum_hash = Sha256::digest(Sha256::digest(
        &address_decoded_bytes[0..(lenght_bytes - 4)],
    ));

    let checksum_address = &address_decoded_bytes[(lenght_bytes - 4)..lenght_bytes];
    if checksum_address != &checksum_hash[..4] {
        return Err(Box::new(std::io::Error::new(
            io::ErrorKind::Other,
            "La dirección es inválida, falló la validación del checksum",
        )));
    }
    Ok(())
}

//      <Sig> <PubKey> OP_DUP OP_HASH160 <PubkeyHash> OP_EQUALVERIFY OP_CHECKSIG
//
// scriptPubKey: OP_DUP OP_HASH160 <bytes_to_push> <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG
// HEXA:         0x76   0xA9       <bytes_to_push> <pubKeyHash>  0x88            0xAC
// Largo bytes:  1 + 1 + 1 + 20 + 1 + 1 = 25
// Si una Tx es P2PKH el largo de su pk_script debe ser == 25

// <pubKeyHash>: Son 20 bytes. Es el resultado de aplicar hash160 (sha256 + ripemd160 hash) a la publicKey comprimida SEC

// scriptSig:   <length sig>     <sig>   <length pubKey>   <pubKey>
// <pubKey> es la publicKey comprimida SEC (33bytes) del receptor de la tx
// Largo bytes: 1 + 72 + 1 + 33 = 107

/// Genera el pk_script de una transaccion P2PKH
/// Recibe el <pubKeyHash> del receptor de la tx.
pub fn generate_p2pkh_pk_script(pubkey_hash: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    if pubkey_hash.len() != 20 {
        return Err(Box::new(std::io::Error::new(
            io::ErrorKind::Other,
            "El pubKey hash recibido es inválido. No tiene el largo correcto",
        )));
    }
    let mut pk_script: Vec<u8> = Vec::new();
    pk_script.push(0x76); // OP_DUP  -> Pasar a constantes o enum
    pk_script.push(0xA9);
    pk_script.push(20); // <bytes_to_push>: Son 20 bytes

    //   let pk = secp256k1::PublicKey::from_slice(public_key).unwrap();
    //    let public_key_sha256_hash = Sha256::digest(public_key);
    //    let public_key_hash160 = *ripemd160::Hash::hash(&public_key_sha256_hash).as_byte_array();

    pk_script.extend_from_slice(pubkey_hash);
    pk_script.push(0x88);
    pk_script.push(0xAC);
    Ok(pk_script)
}

#[cfg(test)]

mod test {
    use std::error::Error;

    use bitcoin_hashes::{ripemd160, Hash};
    use k256::sha2::Digest;
    use k256::sha2::Sha256;
    use secp256k1::SecretKey;

    use crate::account::Account;
    use crate::address_decoder::generate_p2pkh_pk_script;

    use super::get_pubkey_hash_from_address;

    fn generate_pubkey_hash(private_key: &[u8]) -> [u8; 20] {
        let secp: secp256k1::Secp256k1<secp256k1::All> = secp256k1::Secp256k1::new();
        let key: SecretKey = SecretKey::from_slice(private_key).unwrap();
        let public_key: secp256k1::PublicKey = secp256k1::PublicKey::from_secret_key(&secp, &key);
        //  se aplica RIPEMD160(SHA256(ECDSA(public_key)))
        let public_key_compressed = public_key.serialize();
        // let pk_hex: String = public_key_hexa.encode_hex::<String>();

        // Aplica hash160
        let sha256_hash = Sha256::digest(public_key_compressed);
        let ripemd160_hash = *ripemd160::Hash::hash(&sha256_hash).as_byte_array();
        ripemd160_hash
    }

    #[test]
    fn test_decodificacion_de_address_valida_devuelve_ok() {
        let address = "mpzx6iZ1WX8hLSeDRKdkLatXXPN1GDWVaF";
        let pubkey_hash_expected = get_pubkey_hash_from_address(address);
        assert!(pubkey_hash_expected.is_ok())
    }

    #[test]
    fn test_decodificacion_de_adress_genera_pubkey_esperado() -> Result<(), Box<dyn Error>> {
        let address: &str = "mnEvYsxexfDEkCx2YLEfzhjrwKKcyAhMqV";
        let private_key: &str = "cMoBjaYS6EraKLNqrNN8DvN93Nnt6pJNfWkYM8pUufYQB5EVZ7SR";
        let private_key_bytes = Account::decode_wif_private_key(private_key)?;
        let pubkey_hash_expected = generate_pubkey_hash(&private_key_bytes);
        let pubkey_hash_generated = get_pubkey_hash_from_address(address)?;
        assert_eq!(pubkey_hash_expected, pubkey_hash_generated);
        Ok(())
    }

    #[test]
    fn test_pk_script_se_genera_con_el_largo_correcto() -> Result<(), Box<dyn Error>> {
        let pub_key_hash: [u8; 20] = [0; 20];
        let pk_script = generate_p2pkh_pk_script(&pub_key_hash)?;

        assert_eq!(pk_script.len(), 25);
        Ok(())
    }

    #[test]
    fn test_pk_script_se_genera_con_el_contenido_correcto() -> Result<(), Box<dyn Error>> {
        let pub_key_hash: [u8; 20] = [0; 20];
        let pk_script = generate_p2pkh_pk_script(&pub_key_hash)?;

        assert_eq!(pk_script[..1], [0x76]);
        assert_eq!(pk_script[1..2], [0xA9]);
        assert_eq!(pk_script[2..3], [20]);
        assert_eq!(pk_script[3..23], pub_key_hash);
        assert_eq!(pk_script[23..24], [0x88]);
        assert_eq!(pk_script[24..25], [0xAC]);
        Ok(())
    }

    #[test]
    fn test_pub_key_hash_se_genera_con_el_largo_correcto() -> Result<(), Box<dyn Error>> {
        let address = "mnEvYsxexfDEkCx2YLEfzhjrwKKcyAhMqV";
        let pub_key_hash = get_pubkey_hash_from_address(address)?;

        assert_eq!(pub_key_hash.len(), 20);
        Ok(())
    }
    #[test]
    fn test_get_pubkey_hash_con_direccion_invalida_da_error() -> Result<(), Box<dyn Error>> {
        let address = "1nEvYsxexfDEkCx2YLEfzhjrwKKcyAhMqV";
        let pub_key_hash_result = get_pubkey_hash_from_address(address);

        assert!(pub_key_hash_result.is_err());
        Ok(())
    }
    /* todo:
    #[test]
    fn test_pub_key_hash_se_genera_correctamente() -> Result<(), Box<dyn Error>> {
        let address = "mnEvYsxexfDEkCx2YLEfzhjrwKKcyAhMqV";

        let pub_key_hash = get_pubkey_hash_from_address(address)?;
        let private_key = "cMoBjaYS6EraKLNqrNN8DvN93Nnt6pJNfWkYM8pUufYQB5EVZ7SR";
        // se puede crear esta funcion para testearlo. Seguramente haga falta también para validar el script.
        let pub_key_hash_expected = get_pub_key_hash_from_private_key();

        assert_eq!(pub_key_hash, [0x76]);
        Ok(())
    }
    */
}
