//use crate::compact_size_uint::CompactSizeUint;
//todo: CAMBIAR ESE u8 POR compact_size_uint
use std::str::Utf8Error;

#[derive(Clone, Debug)]
pub struct VersionPayload {
    pub version: i32,            // highest protocol version.
    pub services: u64,           // services supported by our node.
    pub timestamp: i64, // The current Unix epoch time according to the transmitting node’s clock.
    pub addr_recv_service: u64, // The services supported by the receiving node as perceived by the transmitting node.
    pub addr_recv_ip: [u8; 16], // The IPv6 address of the receiving node as perceived by the transmitting node in big endian byte order.
    pub addr_recv_port: u16, // The port number of the receiving node as perceived by the transmitting node in big endian byte order.
    pub addr_trans_service: u64, // The services supported by the transmitting node.
    pub addr_trans_ip: [u8; 16], // The IPv6 address of the transmitting node in big endian byte order.
    pub addr_trans_port: u16, // The port number of the transmitting node in big endian byte order.
    pub nonce: u64,           // A random nonce which can help a node detect a connection to itself.
    pub user_agent_bytes: u8, // Number of bytes in following user_agent field.
    pub user_agent: String,   // User agent as defined by BIP14.
    pub start_height: i32, // The height of the transmitting node’s best block chain or, in the case of an SPV client, best block header chain.
    pub relay: bool,       // Transaction relay flag.
}

fn get_version_from_bytes(bytes: &[u8], counter: &mut usize) -> i32 {
    let mut version_bytes = [0; 4];
    version_bytes[..4].copy_from_slice(&bytes[..4]);
    let version = i32::from_le_bytes(version_bytes);
    *counter += 4;
    version
}
fn get_services_from_bytes(bytes: &[u8], counter: &mut usize) -> u64 {
    let mut services_bytes: [u8; 8] = [0; 8];
    services_bytes[..8].copy_from_slice(&bytes[*counter..(8 + *counter)]);
    let services = u64::from_le_bytes(services_bytes);
    *counter += 8;
    services
}

fn get_timestamp_from_bytes(bytes: &[u8], counter: &mut usize) -> i64 {
    let mut timestamp_bytes: [u8; 8] = [0; 8];
    timestamp_bytes[..8].copy_from_slice(&bytes[*counter..(8 + *counter)]);
    let timestamp = i64::from_le_bytes(timestamp_bytes);
    *counter += 8;
    timestamp
}

fn get_addr_services_from_bytes(bytes: &[u8], counter: &mut usize) -> u64 {
    let mut addr_recv_services_bytes: [u8; 8] = [0; 8];
    addr_recv_services_bytes[..8].copy_from_slice(&bytes[*counter..(8 + *counter)]);
    let addr_recv_service = u64::from_le_bytes(addr_recv_services_bytes);
    *counter += 8;
    addr_recv_service
}

fn get_addr_ip_from_bytes(bytes: &[u8], counter: &mut usize) -> [u8; 16] {
    let mut addr_recv_ip: [u8; 16] = [0; 16];
    addr_recv_ip[..16].copy_from_slice(&bytes[*counter..(16 + *counter)]); // ya deberian estar en big endian
    *counter += 16;
    addr_recv_ip
}

fn get_addr_port_from_bytes(bytes: &[u8], counter: &mut usize) -> u16 {
    let mut addr_recv_port_bytes: [u8; 2] = [0; 2];
    addr_recv_port_bytes[..2].copy_from_slice(&bytes[*counter..(2 + *counter)]);
    let addr_recv_port = u16::from_be_bytes(addr_recv_port_bytes);
    *counter += 2;
    addr_recv_port
}

fn get_nonce_from_bytes(bytes: &[u8], counter: &mut usize) -> u64 {
    let mut nonce_bytes: [u8; 8] = [0; 8];
    nonce_bytes[..8].copy_from_slice(&bytes[*counter..(8 + *counter)]);
    let nonce = u64::from_le_bytes(nonce_bytes);
    *counter += 8;
    nonce
}

fn get_user_agent_bytes_from_bytes(bytes: &[u8], counter: &mut usize) -> u8 {
    let mut u_agent_bytes: [u8; 1] = [0; 1];
    u_agent_bytes[..1].copy_from_slice(&bytes[*counter..(1 + *counter)]);
    let user_agent_bytes = u8::from_le_bytes(u_agent_bytes);
    *counter += 1;
    user_agent_bytes
}

fn get_start_height_from_bytes(bytes: &[u8], counter: &mut usize) -> i32 {
    let mut start_height_bytes: [u8; 4] = [0; 4];
    start_height_bytes[..4].copy_from_slice(&bytes[*counter..(4 + *counter)]);
    let start_height = i32::from_le_bytes(start_height_bytes);
    *counter += 4;
    start_height
}

fn get_relay_from_bytes(bytes: &[u8], counter: usize) -> bool {
    let relay_byte = bytes[counter];
    matches!(relay_byte, 1u8)
}

fn get_user_agent_from_bytes(
    bytes: &[u8],
    counter: &mut usize,
    user_agent_bytes: u8,
) -> Result<String, Utf8Error> {
    let mut user_agent_b = vec![0; user_agent_bytes as usize];
    user_agent_b.copy_from_slice(&bytes[*counter..(user_agent_bytes as usize + *counter)]);
    let user_agent = std::str::from_utf8(&user_agent_b)?.to_string();
    *counter += user_agent_bytes as usize;
    Ok(user_agent)
}

impl VersionPayload {
    pub fn to_le_bytes(&self) -> Vec<u8> {
        let mut version_payload_bytes: Vec<u8> = vec![];
        version_payload_bytes.extend_from_slice(&self.version.to_le_bytes());
        version_payload_bytes.extend_from_slice(&self.services.to_le_bytes());
        version_payload_bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        version_payload_bytes.extend_from_slice(&self.addr_recv_service.to_le_bytes());
        version_payload_bytes.extend_from_slice(&self.addr_recv_ip); // big endian bytes
        version_payload_bytes.extend_from_slice(&self.addr_recv_port.to_be_bytes()); // big endian bytes
        version_payload_bytes.extend_from_slice(&self.addr_trans_service.to_le_bytes());
        version_payload_bytes.extend_from_slice(&self.addr_trans_ip); // big endian bytes
        version_payload_bytes.extend_from_slice(&self.addr_trans_port.to_be_bytes()); // big endian bytes
        version_payload_bytes.extend_from_slice(&self.nonce.to_le_bytes());
        version_payload_bytes.extend_from_slice(&self.user_agent_bytes.to_le_bytes());
        version_payload_bytes.extend_from_slice(self.user_agent.as_bytes()); // little -> depende arq de computadora ??
        version_payload_bytes.extend_from_slice(&self.start_height.to_le_bytes());
        version_payload_bytes.push(self.relay as u8);
        version_payload_bytes
    }

    pub fn from_le_bytes(bytes: &[u8]) -> Result<Self, Utf8Error> {
        let mut counter = 0;

        let version = get_version_from_bytes(bytes, &mut counter);
        let services = get_services_from_bytes(bytes, &mut counter);
        let timestamp = get_timestamp_from_bytes(bytes, &mut counter);
        let addr_recv_service = get_addr_services_from_bytes(bytes, &mut counter);
        let addr_recv_ip = get_addr_ip_from_bytes(bytes, &mut counter);
        let addr_recv_port = get_addr_port_from_bytes(bytes, &mut counter);
        let addr_trans_service = get_addr_services_from_bytes(bytes, &mut counter);
        let addr_trans_ip = get_addr_ip_from_bytes(bytes, &mut counter);
        let addr_trans_port = get_addr_port_from_bytes(bytes, &mut counter);
        let nonce = get_nonce_from_bytes(bytes, &mut counter);
        let user_agent_bytes = get_user_agent_bytes_from_bytes(bytes, &mut counter);
        let user_agent = get_user_agent_from_bytes(bytes, &mut counter, user_agent_bytes)?;
        let start_height = get_start_height_from_bytes(bytes, &mut counter);
        let relay = get_relay_from_bytes(bytes, counter);
        Ok(VersionPayload {
            version,
            services,
            timestamp,
            addr_recv_service,
            addr_recv_ip,
            addr_recv_port,
            addr_trans_service,
            addr_trans_ip,
            addr_trans_port,
            nonce,
            user_agent_bytes,
            user_agent,
            start_height,
            relay,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /*
        #[test]
        fn a_version_payload_is_correctly_converted_to_le_bytes() {
                let version = PROTOCOL_VERSION;
                let services: u64 = 0;
                let timestamp: i64 = match get_current_unix_epoch_time() {
                    Err(e) => {
                        println!("ERROR: {}", e);
                        exit(-1)
                    },
                    Ok(timestamp) => timestamp,
                };
                let addr_recv_service: u64 = 1;
                let addr_recv_ip = get_ipv6_address_ip(socket_addr);
                let addr_recv_port: u16 = 18333;
                let addr_trans_service: u64 = 0;
                let addr_trans_ip = get_ipv6_address_ip(local_ip_addr);
                let addr_trans_port: u16 = 18333;
                let nonce: u64 = rand::thread_rng().gen();
                let user_agent_bytes: u8 = 15u8; // ??????
                let user_agent: String = "/Satoshi:23.0.0/".to_string();
                let start_height: i32 = 1;
                let relay: bool = true;
                let version_payload = VersionPayload {
                    version,
                    services,
                    timestamp,
                    addr_recv_service,
                    addr_recv_ip,
                    addr_recv_port,
                    addr_trans_service,
                    addr_trans_ip,
                    addr_trans_port,
                    nonce,
                    user_agent_bytes,
                    user_agent,
                    start_height,
                    relay,
                };
            }
    */
}
