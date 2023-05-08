use crate::compact_size_uint::CompactSizeUint;

#[derive(Clone, Debug)]
/// Representa el payload del mensaje getheaders segun el protocolo de bitcoin
pub struct GetHeadersPayload {
    pub version: u32, // The protocol version
    pub hash_count: CompactSizeUint,
    pub locator_hashes: Vec<[u8; 32]>, // Locator hashes — ordered newest to oldest. The remote peer will reply with its longest known chain, starting from a locator hash if possible and block 1 otherwise.
    pub stop_hash: [u8; 32], // References the header to stop at, or zero to just fetch the maximum 2000 headers
}

impl GetHeadersPayload {
    /// Dado un struct del tipo GetHeadersPayload serializa el payload a bytes segun el protocolo de bitcoin
    /// y devuelve un vetor de bytes que representan el payload del mensaje getheaders
    pub fn to_le_bytes(&self) -> Vec<u8> {
        let mut getheaders_payload_bytes: Vec<u8> = vec![];
        getheaders_payload_bytes.extend_from_slice(&self.version.to_le_bytes());
        getheaders_payload_bytes.extend_from_slice(&self.hash_count.marshalling());
        for hash in &self.locator_hashes {
            getheaders_payload_bytes.extend(hash);
        }
        getheaders_payload_bytes.extend(self.stop_hash);
        getheaders_payload_bytes
    }
}




