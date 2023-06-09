use bitcoin_hashes::sha256;
use bitcoin_hashes::{ripemd160, Hash};
use k256::ecdsa::{signature::Signer, signature::Verifier, Signature, SigningKey, VerifyingKey};
use k256::elliptic_curve::generic_array::GenericArray;
use k256::elliptic_curve::SecretKey;
//use secp256k1::hashes::sha256;

#[derive(Debug, PartialEq, Clone)]
pub struct SigScript {
    bytes: Vec<u8>,
}

impl SigScript {
    pub fn new(bytes: Vec<u8>) -> Self {
        SigScript { bytes }
    }

    pub fn get_bytes(&self) -> &Vec<u8> {
        &self.bytes
    }

    /// Recibe el hash a firmar y la private key
    /// Devuelve el signature
    pub fn generate_sig(hash: [u8; 32], private_key: [u8; 32]) -> Vec<u8> {
        let signature = Signature::from_scalars(hash, private_key).unwrap();

        //let signature: Signature = signing_key.sign(hash);
        let bytes_signature = signature.to_der().to_bytes();
        let bytes = bytes_signature.to_vec();
        bytes
    }
}

#[cfg(test)]

mod test {
    use hex::ToHex;

    use crate::transactions::sig_script::SigScript;
    #[test]
    fn test_el_largo_del_script_sig_es_72_bytes() {
        let hash: [u8; 32] = [123; 32];
        let signing_key: [u8; 32] = [14; 32];

        let sig = SigScript::generate_sig(hash, signing_key);
        let height_hex: String = sig.encode_hex::<String>();
        println!("height :{}", height_hex);

        assert_eq!(sig.len(), 72)
    }
}
