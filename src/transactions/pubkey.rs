use std::error::Error;

use k256::sha2::Digest;
use k256::sha2::Sha256;

use crate::address_decoder::get_pubkey_hash_from_address;

#[derive(Debug, PartialEq, Clone)]
pub struct Pubkey {
    bytes: Vec<u8>,
}

impl Pubkey {
    pub fn new(bytes: Vec<u8>) -> Self {
        Pubkey { bytes }
    }
    pub fn bytes(&self) -> &Vec<u8> {
        &self.bytes
    }
    pub fn generate_adress(&self) -> Result<String, &'static str> {
        // vector que generara el adress
        let mut adress_bytes: Vec<u8> = vec![0x6f];
        let bytes = &self.bytes;
        let lenght: usize = bytes.len();
        if lenght <= 3 {
            return Err("el campo pubkey no tiene el largo esperado");
        }

        let first_byte = self.bytes[0];
        if first_byte == 0x00 {
            // se trata de una transanccion del tipo P2WPKH
            adress_bytes.extend_from_slice(&bytes[2..lenght]);
        }
        if first_byte == 0x76 {
            // se trata de una transanccion del tipo P2PKH
            adress_bytes.extend_from_slice(&bytes[3..(lenght - 2)]);
        }
        //println!("{:?}", adress_bytes);
        let copy_adress_bytes: Vec<u8> = adress_bytes.clone();
        let checksum = Sha256::digest(Sha256::digest(copy_adress_bytes));
        adress_bytes.extend_from_slice(&checksum[..4]);
        let encoded: bs58::encode::EncodeBuilder<&Vec<u8>> = bs58::encode(&adress_bytes);
        let string = encoded.into_string();
        Ok(string)
    }

    pub fn generate_pubkey(address: &str) -> Result<Pubkey, Box<dyn Error>> {
        let pubkey_hash = get_pubkey_hash_from_address(address)?;
        let mut pk_script: Vec<u8> = Vec::new();
        pk_script.push(0x76); // OP_DUP  -> Pasar a constantes o enum
        pk_script.push(0xA9);
        pk_script.push(20); // <bytes_to_push>: Son 20 bytes
        pk_script.extend_from_slice(&pubkey_hash);
        pk_script.push(0x88);
        pk_script.push(0xAC);
        Ok(Pubkey { bytes: pk_script })
    }
}
