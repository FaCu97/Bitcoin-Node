use k256::ecdsa::Signature;
use std::error::Error;

use crate::user::User;

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
        let bytes_signature = signature.to_der().to_bytes();
        let bytes = bytes_signature.to_vec();
        bytes
    }
    ///funcion que devuelve el signature script con la clave publica comprimida
    pub fn generate_sig_script(
        hash_transaction: [u8; 32],
        user: User,
    ) -> Result<SigScript, Box<dyn Error>> {
        let mut sig_script_bytes: Vec<u8> = Vec::new();
        let private_key = user.get_private_key()?;
        let sig = Self::generate_sig(hash_transaction, private_key);
        let lenght_sig = sig.len();
        // esto equivale al op inicial que indica el largo del campo sig
        sig_script_bytes.push(lenght_sig as u8);
        // se carga el campo sig
        sig_script_bytes.extend_from_slice(&sig);
        let bytes_public_key = user.get_pubkey_compressed()?;
        let lenght_pubkey = bytes_public_key.len();
        // se carga el largo de los bytes de la clave publica
        sig_script_bytes.push(lenght_pubkey as u8);
        // se carga la clave publica comprimida (sin hashear)
        sig_script_bytes.extend_from_slice(&bytes_public_key);
        let sig_script = Self::new(sig_script_bytes);
        Ok(sig_script)
    }
}

#[cfg(test)]

mod test {
    use crate::transactions::sig_script::SigScript;
    #[test]
    fn test_el_largo_del_script_sig_es_70_bytes() {
        let hash: [u8; 32] = [123; 32];
        let signing_key: [u8; 32] = [14; 32];

        let sig = SigScript::generate_sig(hash, signing_key);
        assert_eq!(sig.len(), 70)
    }
}
