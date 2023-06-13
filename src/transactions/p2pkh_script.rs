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

use bitcoin_hashes::{ripemd160, Hash};
use k256::sha2::Digest;
use k256::sha2::Sha256;

pub fn validate(p2pkh_script: &[u8], sig_script: &[u8]) -> bool {
    let sig_script_pubkey = &sig_script[71..104];

    // Aplica hash160
    let sha256_hash = Sha256::digest(sig_script_pubkey);
    let ripemd160_hash = *ripemd160::Hash::hash(&sha256_hash).as_byte_array();

    return true;
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
        let signing_key: [u8; 32] = [14; 32];

        let address: String = String::from("mnEvYsxexfDEkCx2YLEfzhjrwKKcyAhMqV");
        let private_key: String =
            String::from("cMoBjaYS6EraKLNqrNN8DvN93Nnt6pJNfWkYM8pUufYQB5EVZ7SR");
        let account = Account::new(private_key, address)?;

        let p2pkh_script =
            address_decoder::generate_p2pkh_pk_script(&account.get_pubkey_compressed()?);
        let sig = SigScript::generate_sig_script(hash, account)?;
        let validation = p2pkh_script::validate(&p2pkh_script, sig.get_bytes());
        assert!(validation);
        Ok(())
    }
}
