use crate::account::Account;
use k256::ecdsa;
use k256::ecdsa::Signature;
use k256::ecdsa::SigningKey;
use k256::ecdsa::VerifyingKey;
use k256::elliptic_curve;
use k256::elliptic_curve::scalar::Scalar;
use k256::elliptic_curve::sec1::FromEncodedPoint;
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::elliptic_curve::AffinePoint;
use k256::schnorr::signature;
use k256::schnorr::signature::SignatureEncoding;
use k256::schnorr::signature::Signer;
use k256::schnorr::signature::Verifier;
use k256::EncodedPoint;
use k256::ProjectivePoint;
use std::convert::TryInto;
use std::error::Error;
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
    fn generate_sig(hash: [u8; 32], private_key: [u8; 32]) -> Vec<u8> {
        // Signing
        let secret_key = elliptic_curve::SecretKey::from_bytes((&private_key).into()).unwrap();
        let signing_key = ecdsa::SigningKey::from(secret_key);
        let signature: ecdsa::Signature = signing_key.sign(&hash);
        let signature_bytes: Vec<u8> = signature.to_der().to_vec();
        signature_bytes
    }

    ///funcion que devuelve el signature script con la clave publica comprimida
    pub fn generate_sig_script(
        hash_transaction: [u8; 32],
        account: Account,
    ) -> Result<SigScript, Box<dyn Error>> {
        let mut sig_script_bytes: Vec<u8> = Vec::new();
        let private_key = account.get_private_key()?;
        let sig = Self::generate_sig(hash_transaction, private_key);
        let lenght_sig = sig.len();
        // esto equivale al op inicial que indica el largo del campo sig
        sig_script_bytes.push(lenght_sig as u8);
        // se carga el campo sig
        sig_script_bytes.extend_from_slice(&sig);
        let bytes_public_key = account.get_pubkey_compressed()?;
        let lenght_pubkey = bytes_public_key.len();
        // se carga el largo de los bytes de la clave publica
        sig_script_bytes.push(lenght_pubkey as u8);
        // se carga la clave publica comprimida (sin hashear)
        sig_script_bytes.extend_from_slice(&bytes_public_key);
        let sig_script = Self::new(sig_script_bytes);
        Ok(sig_script)
    }

    /// Recive el hash, signature y public key.
    /// Devuelve true o false dependiendo si el signature es correcto.
    pub fn verify_sig(hash: [u8; 32], signature_bytes: &[u8], public_key: &[u8]) -> bool {
        // Verifying
        let verifying_key = ecdsa::VerifyingKey::from_sec1_bytes(public_key).unwrap();
        let signature = ecdsa::Signature::from_der(signature_bytes).unwrap();
        let verified = verifying_key.verify(&hash, &signature).is_ok();

        verified
    }
}
#[cfg(test)]
mod test {
    use std::error::Error;

    use crate::{account::Account, transactions::sig_script::SigScript};
    #[test]
    fn test_el_largo_del_script_sig_es_70_bytes() {
        let hash: [u8; 32] = [123; 32];
        let signing_key: [u8; 32] = [14; 32];

        let sig = SigScript::generate_sig(hash, signing_key);
        assert_eq!(sig.len(), 70)
    }

    #[test]
    fn test_la_firma_se_realiza_correctamente() -> Result<(), Box<dyn Error>> {
        let hash: [u8; 32] = [123; 32];
        let address_expected: String = String::from("mnEvYsxexfDEkCx2YLEfzhjrwKKcyAhMqV");
        let private_key: String =
            String::from("cMoBjaYS6EraKLNqrNN8DvN93Nnt6pJNfWkYM8pUufYQB5EVZ7SR");
        let account = Account::new(private_key, address_expected)?;
        let sig = SigScript::generate_sig(hash.clone(), account.get_private_key()?);
        assert!(SigScript::verify_sig(
            hash,
            &sig,
            &account.get_pubkey_compressed()?
        ));
        Ok(())
    }
}
