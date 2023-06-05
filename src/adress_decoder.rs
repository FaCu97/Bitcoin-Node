use k256::sha2::Digest;
use k256::sha2::Sha256;

pub fn decode_address(address: &str) -> Result<[u8; 20], &'static str> {
    //se decodifican de &str a bytes , desde el formate base58  a bytes
    let decoded_bytes = bs58::decode(address).into_vec();
    let bytes = match decoded_bytes {
        Ok(value) => value,
        Err(_) => return Err("fallo la decodificacion en base58"),
    };
    //generacion del checksum
    let lenght_bytes = bytes.len();
    let mut pubkey_hash = vec![0x6f];
    pubkey_hash.extend_from_slice(&bytes[1..(lenght_bytes - 4)]);
    let checksum = Sha256::digest(Sha256::digest(&pubkey_hash));
    // se guarda el checksum esperado
    let mut checksum_expected: Vec<u8> = Vec::new();
    checksum_expected.extend_from_slice(&checksum[..4]);
    // se obtiene el checksum recibido
    let mut checksum_received = Vec::new();
    checksum_received.extend_from_slice(&bytes[(lenght_bytes - 4)..lenght_bytes]);
    if checksum_received != checksum_expected {
        return Err("el checksum recibido no es el esperado");
    }
    // pubkey-hash
    let mut pubkey_hash_to_return = [0; 20];
    pubkey_hash_to_return.copy_from_slice(&bytes[1..(lenght_bytes - 4)]);
    Ok(pubkey_hash_to_return)
}
