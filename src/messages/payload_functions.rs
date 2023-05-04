use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};
use std::net::SocketAddr;


pub fn get_current_unix_epoch_time() -> Result<i64, SystemTimeError> {
    let current_time = SystemTime::now();
    let unix_epoch = UNIX_EPOCH;
    let unix_time = current_time.duration_since(unix_epoch)?;
    let seconds = unix_time.as_secs() as i64;
    Ok(seconds)
}

pub fn get_ipv6_address_ip(socket_addr: SocketAddr) -> [u8; 16] {
    let mut addr_recv_ip: [u8; 16] = [0; 16];
    let addr_recv_ip_aux: [u16; 8] = match socket_addr {
        SocketAddr::V4(addr) => addr.ip().to_ipv6_mapped().segments(),
        SocketAddr::V6(addr) => addr.ip().segments(),
    };
    for (i, num) in addr_recv_ip_aux.iter().enumerate() {
        let bytes = num.to_be_bytes(); // convertimos a bytes en orden de bytes de menor a mayor
        addr_recv_ip[(i * 2)..(i * 2 + 2)].copy_from_slice(&bytes); // copiamos los bytes en el vector de 8 bits
    }
    addr_recv_ip
}