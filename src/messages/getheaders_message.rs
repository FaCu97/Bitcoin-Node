pub struct GetHeadersPayload {
    pub version: u32, // The protocol version
    pub locator_hashes: Vec<[u8; 32]>, // Locator hashes — ordered newest to oldest. The remote peer will reply with its longest known chain, starting from a locator hash if possible and block 1 otherwise.
    pub stop_hash: [u8; 32], // References the header to stop at, or zero to just fetch the maximum 2000 headers
}