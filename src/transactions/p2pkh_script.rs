//      <Sig> <PubKey> OP_DUP OP_HASH160 <PubkeyHash> OP_EQUALVERIFY OP_CHECKSIG
//
// scriptPubKey: OP_DUP OP_HASH160 <bytes_to_push> <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG
// HEXA:         0x76   0xA9       <bytes_to_push> <pubKeyHash>  0x88            0xAC
// Largo bytes:  1 + 1 + 1 + 20 + 1 + 1 = 25
// Si una Tx es P2PKH el largo de su pk_script debe ser == 25

// <pubKeyHash>: Son 20 bytes. Es el resultado de aplicar hash160 (sha256 + ripemd160 hash) a la publicKey comprimida SEC

// scriptSig:   <length sig>     <sig>   <length pubKey>   <pubKey>
// <pubKey> es la publicKey comprimida SEC (33bytes) del receptor de la tx
// Largo bytes: 1 + 70 + 1 + 33 = 105

use bitcoin_hashes::{ripemd160, Hash};
use k256::sha2::Digest;
use k256::sha2::Sha256;

/// Recibe el p2pkh_script y el sig_script.
/// Realiza la validación y devuelve true o false
pub fn validate(p2pkh_script: &[u8], sig_script: &[u8]) -> bool {
    let mut sig_script_pubkey: [u8; 33] = [0; 33];
    sig_script_pubkey.copy_from_slice(&sig_script[72..105]);

    // 1) Chequeo que el primer comando sea OP_DUP (0x76)
    if p2pkh_script[0..1] != [0x76] {
        return false;
    }

    // 2) Chequeo que el siguiente comando sea OP_HASH_160 (0xA9)
    if p2pkh_script[1..2] != [0xA9] {
        return false;
    }

    // 3) Aplica hash160 sobre el pubkey del sig_script
    let sha256_hash = Sha256::digest(sig_script_pubkey);
    let ripemd160_hash = *ripemd160::Hash::hash(&sha256_hash).as_byte_array();
    // 4) Chequeo que el siguiente comando sea OP_EQUALVERIFY (0x88)
    if p2pkh_script[23..24] != [0x88] {
        return false;
    }

    // 5) Chequeo que los hash coincidan
    if p2pkh_script[3..23] != ripemd160_hash {
        return false;
    }

    // 6) Chequeo que el siguiente comando sea OP_CHECKSIG (0xAC)
    if p2pkh_script[24..25] != [0xAC] {
        return false;
    }

    true
}

#[cfg(test)]
mod test {
    use std::error::Error;

    use crate::{
        account::Account,
        address_decoder,
        transactions::{p2pkh_script, sig_script::SigScript},
    };
    #[test]
    fn test_p2pkh_script_se_valida_correctamente() -> Result<(), Box<dyn Error>> {
        let hash: [u8; 32] = [123; 32];

        let address: String = String::from("mnEvYsxexfDEkCx2YLEfzhjrwKKcyAhMqV");
        let private_key: String =
            String::from("cMoBjaYS6EraKLNqrNN8DvN93Nnt6pJNfWkYM8pUufYQB5EVZ7SR");
        let account = Account::new(private_key, address)?;

        let p2pkh_script = address_decoder::generate_p2pkh_pk_script(
            &address_decoder::get_pubkey_hash_from_address(&account.address)?,
        )?;
        let sig = SigScript::generate_sig_script(hash, account)?;
        let validation = p2pkh_script::validate(&p2pkh_script, sig.get_bytes());

        assert!(validation);
        Ok(())
    }
}
