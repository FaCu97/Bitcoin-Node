use bitcoin_hashes::{sha256d, Hash};
use std::any::Any;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::str::Utf8Error;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::compact_size_uint::CompactSizeUint;
use crate::listener::handle_ping_message;
use crate::logwriter::log_writer::{write_in_log, LogSender};
use crate::messages::{inventory, payload};

use super::inventory::Inventory;
// todo: implementar test de read_from usando mocking
// todo: implementar test de write_to usando mocking
// todo: implementar test de write_verack_message, read_verack_message, write_sendheaders_message usando mocking
#[derive(Clone, Debug)]
/// Representa el header de cualquier mensaje del protocolo bitcoin
pub struct HeaderMessage {
    pub start_string: [u8; 4],
    pub command_name: String,
    pub payload_size: u32,
    pub checksum: [u8; 4],
}

impl HeaderMessage {
    /// Convierte el struct que representa el header de cualquier mensaje a bytes segun las reglas de
    /// serializacion del protocolo bitcoin
    pub fn to_le_bytes(&self) -> [u8; 24] {
        let mut header_message_bytes: [u8; 24] = [0; 24];
        header_message_bytes[0..4].copy_from_slice(&self.start_string);
        header_message_bytes[4..16].copy_from_slice(&command_name_to_bytes(&self.command_name));
        header_message_bytes[16..20].copy_from_slice(&self.payload_size.to_le_bytes());
        header_message_bytes[20..24].copy_from_slice(&self.checksum);
        header_message_bytes
    }
    /// recibe los bytes de un header de un mensaje y los convierte a un struct HeaderMessage
    /// de acuerdo al protocolo de bitcoin
    pub fn from_le_bytes(bytes: [u8; 24]) -> Result<Self, Utf8Error> {
        let mut start_string = [0; 4];
        let mut counter = 0;
        start_string[..4].copy_from_slice(&bytes[..4]);
        counter += 4;
        let mut command_name_bytes = [0; 12];
        command_name_bytes[..12].copy_from_slice(&bytes[counter..(12 + counter)]);
        counter += 12;
        let command_name = std::str::from_utf8(&command_name_bytes)?.to_string();
        let mut payload_size_bytes: [u8; 4] = [0; 4];
        payload_size_bytes[..4].copy_from_slice(&bytes[counter..(4 + counter)]);
        counter += 4;
        let payload_size = u32::from_le_bytes(payload_size_bytes);
        let mut checksum = [0; 4];
        checksum[..4].copy_from_slice(&bytes[counter..(4 + counter)]);
        Ok(HeaderMessage {
            start_string,
            command_name,
            payload_size,
            checksum,
        })
    }
    /// recibe un struct HeaderMessage que representa un el header de un mensaje segun protocolo de bitcoin
    /// y un stream que implemente el trait Write (en donde se pueda escribir) y escribe el mensaje serializado
    /// en bytes en el stream. Devuelve un error en caso de que no se haya podido escribir correctamente o un Ok en caso
    /// de que se haya escrito correctamente
    pub fn write_to(&self, stream: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>> {
        let header = self.to_le_bytes();
        stream.write_all(&header)?;
        stream.flush()?;
        Ok(())
    }
    /// Recibe un stream que implemente el trait read (algo desde lo que se pueda leer) y el nombre del comando que se quiere leer
    /// y devuelve un HeaderMessage si se pudo leer correctamente uno desde el stream
    /// o Error si lo leido no corresponde a el header de un mensaje del protocolo de bitcoin
    pub fn read_from(
        log_sender: LogSender,
        mut stream: &mut TcpStream,
        command_name: String,
        finish: Option<Arc<RwLock<bool>>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if command_name == *"block" {
            // will wait a minimum of two more seconds for the stalling node to send the block.
            // If the block still hasn’t arrived, Bitcoin Core will disconnect from the stalling
            // node and attempt to connect to another node.
            stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        }
        let header_command_name =
            std::str::from_utf8(&command_name_to_bytes(&command_name))?.to_string();
        let mut buffer_num = [0; 24];
        stream.read_exact(&mut buffer_num)?;
        let mut header = HeaderMessage::from_le_bytes(buffer_num)?;
        // si no se leyo el header que se queria, sigo leyendo hasta encontrarlo
        while header.command_name != header_command_name && !is_terminated(finish.clone()) {
            if header.command_name.contains("ping") {
                write_in_log(
                    log_sender.messege_log_sender.clone(),
                    format!(
                        "Recibo Correctamente: ping -- Nodo: {:?}",
                        stream.peer_addr()?
                    )
                    .as_str(),
                );
                let payload_bytes = read_payload(&mut stream, &header)?;
                write_pong_message(&mut stream, &payload_bytes)?;
            } 
            if header.command_name.contains("inv") {
                write_in_log(
                    log_sender.messege_log_sender.clone(),
                    format!(
                        "Recibo Correctamente: inv -- Nodo: {:?}",
                        stream.peer_addr()?
                    )
                    .as_str(),
                );
                let payload_bytes = read_payload(&mut stream, &header)?;
                handle_inv_message(payload_bytes);
            } else {
                write_in_log(
                    log_sender.messege_log_sender.clone(),
                    format!(
                        "IGNORADO -- Recibo: {} -- Nodo: {:?}",
                        header.command_name,
                        stream.peer_addr()?
                    )
                    .as_str(),
                );
                read_payload(&mut stream, &header)?;
            }
            buffer_num = [0; 24];
            stream.read_exact(&mut buffer_num)?;
            header = HeaderMessage::from_le_bytes(buffer_num)?;
        }
        if !is_terminated(finish) {
            write_in_log(
                log_sender.messege_log_sender,
                format!(
                    "Recibo Correctamente: {} -- Nodo: {:?}",
                    command_name,
                    stream.peer_addr()?
                )
                .as_str(),
            );
        }
        Ok(header)
    }
}

pub fn is_terminated(finish: Option<Arc<RwLock<bool>>>) -> bool {
    match finish {
        Some(m) => *m.read().unwrap(),
        None => false,
    }
}

fn read_payload(stream: &mut dyn Read, header: &HeaderMessage) -> io::Result<Vec<u8>> {
    let payload_size = header.payload_size as usize;
    let mut payload_buffer_num: Vec<u8> = vec![0; payload_size];
    stream.read_exact(&mut payload_buffer_num)?;
    Ok(payload_buffer_num)
}

fn handle_inv_message(payload_bytes: Vec<u8>) {
    let mut offset: usize = 0;
    let count = CompactSizeUint::unmarshalling(&payload_bytes, &mut offset).unwrap();
    for _ in 0..count.decoded_value() as usize {
        let mut inventory_bytes = vec![0; 36];
        inventory_bytes.copy_from_slice(&payload_bytes[offset..(offset + 36)]);
        let inv = Inventory::from_le_bytes(&inventory_bytes);
        println!("{:?}", inv);
        offset += 36;
    }

}

/// Recibe un stream que implemente el trait Write (algo donde se pueda escribir) y escribe el mensaje verack segun
/// el protocolo de bitcoin, si se escribe correctamente devuelve Ok(()) y sino devuelve un error
pub fn write_verack_message(stream: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>> {
    let header = HeaderMessage {
        start_string: [0x0b, 0x11, 0x09, 0x07],
        command_name: "verack".to_string(),
        payload_size: 0,
        checksum: [0x5d, 0xf6, 0xe0, 0xe2], // checksum de payload vacio
    };
    header.write_to(stream)?;
    Ok(())
}

/// Recibe un stream que implemente el trait Write (algo donde se pueda escribir) y el nonce del mensaje ping
/// al que le tiene que responder y escribe el mensaje pong segun
/// el protocolo de bitcoin, si se escribe correctamente devuelve Ok(()) y sino devuelve un error
pub fn write_pong_message(
    stream: &mut dyn Write,
    payload: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let checksum = get_checksum(payload);
    let header = HeaderMessage {
        start_string: [0x0b, 0x11, 0x09, 0x07],
        command_name: "pong".to_string(),
        payload_size: payload.len() as u32,
        checksum,
    };
    let header_bytes = HeaderMessage::to_le_bytes(&header);
    let mut message: Vec<u8> = Vec::new();
    message.extend_from_slice(&header_bytes);
    message.extend(payload);
    stream.write_all(&message)?;
    stream.flush()?;
    Ok(())
}

/// Recibe un stream que implemente el trait Write (algo donde se pueda escribir) y escribe el mensaje sendheaders segun
/// el protocolo de bitcoin, si se escribe correctamente devuelve Ok(()) y sino devuelve un error
pub fn write_sendheaders_message(stream: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>> {
    let header = HeaderMessage {
        start_string: [0x0b, 0x11, 0x09, 0x07],
        command_name: "sendheaders".to_string(),
        payload_size: 0,
        checksum: [0x5d, 0xf6, 0xe0, 0xe2], // checksum de payload vacio
    };
    header.write_to(stream)?;
    Ok(())
}
/// Recibe un stream que implemente el trait Read (algo donde se pueda Leer) y lee el mensaje verack segun
/// el protocolo de bitcoin, si se lee correctamente devuelve Ok(HeaderMessage) y sino devuelve un error
pub fn read_verack_message(
    log_sender: LogSender,
    stream: &mut TcpStream,
) -> Result<HeaderMessage, Box<dyn std::error::Error>> {
    HeaderMessage::read_from(log_sender, stream, "verack".to_string(), None)
}

/// Recibe un String que representa el nombre del comando del Header Message
/// y devuelve los bytes que representan ese string (ASCII) seguido de 0x00 para
/// completar los 12 bytes
/// little-endian
pub fn command_name_to_bytes(command: &String) -> [u8; 12] {
    let mut command_name_bytes = [0; 12];
    let command_bytes = command.as_bytes();
    command_name_bytes[..command_bytes.len()]
        .copy_from_slice(&command_bytes[..command_bytes.len()]);
    command_name_bytes
}

pub fn get_checksum(payload: &[u8]) -> [u8; 4] {
    let sha_hash = sha256d::Hash::hash(payload); // hasheo doble de los bytes del payload
    let hash_bytes: [u8; 32] = sha_hash.to_byte_array(); // convert Hash to [u8; 32] array
    let mut checksum: [u8; 4] = [0u8; 4];
    checksum.copy_from_slice(&hash_bytes[0..4]); // checksum devuelve los primeros 4 bytes de SHA256(SHA256(payload))
    checksum
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_message_bytes_from_verack_message_unmarshalling_correctly() {
        // GIVEN : un header messege del mensaje verack en bytes
        let header_message_bytes: [u8; 24] = [
            11, 17, 9, 7, 118, 101, 114, 97, 99, 107, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 93, 246, 224,
            226,
        ];
        // WHEN: se ejecuta la funcion form_le_bytes del struct HeaderMessage con los bytes pasados por parametro
        let header = HeaderMessage::from_le_bytes(header_message_bytes).unwrap();
        // THEN: se devuelve un struct HeaderMessage con los campos correctos segun el mensaje verack
        assert_eq!([11u8, 17u8, 9u8, 7u8], header.start_string);
        assert_eq!("verack\0\0\0\0\0\0", header.command_name);
        assert_eq!(0, header.payload_size);
        assert_eq!([93u8, 246u8, 224u8, 226u8], header.checksum);
    }
    #[test]
    fn header_message_bytes_from_version_message_unmarshalling_correctly() {
        // GIVEN : un header messege del mensaje version en bytes
        let header_message_bytes: [u8; 24] = [
            11, 17, 9, 7, 118, 101, 114, 115, 105, 111, 110, 0, 0, 0, 0, 0, 100, 0, 0, 0, 152, 16,
            0, 0,
        ];
        // WHEN: se ejecuta la funcion form_le_bytes del struct HeaderMessage con los bytes pasados por parametro
        let header = HeaderMessage::from_le_bytes(header_message_bytes).unwrap();
        // THEN: se devuelve un struct HeaderMessage con los campos correctos segun el mensaje version
        assert_eq!([11u8, 17u8, 9u8, 7u8], header.start_string);
        assert_eq!("version\0\0\0\0\0", header.command_name);
        assert_eq!(100, header.payload_size);
        assert_eq!([152u8, 16u8, 0u8, 0u8], header.checksum);
    }
    #[test]
    fn error_when_command_name_bytes_can_not_be_represented_as_string() {
        // GIVEN : un header messege de un  mensaje con command name erroneo en bytes
        let header_message_bytes: [u8; 24] = [
            11, 17, 9, 7, 12, 101, 114, 13, 240, 111, 110, 1, 0, 0, 0, 11, 100, 0, 0, 0, 152, 16,
            0, 0,
        ];
        // WHEN: se ejecuta la funcion form_le_bytes del struct HeaderMessage con los bytes pasados por parametro
        let header = HeaderMessage::from_le_bytes(header_message_bytes);
        // THEN: header es un error
        assert!(header.is_err());
        assert!(matches!(header, Err(_)));
    }
    #[test]
    fn header_message_of_a_verack_message_marshalling_correctly_to_bytes() {
        // GIVEN: un struct HeaderMessage de un mensaje verack
        let verack_header_message = HeaderMessage {
            start_string: [11, 17, 9, 7],
            command_name: "verack".to_string(),
            payload_size: 0,
            checksum: [93, 246, 224, 226],
        };
        // WHEN: se ejecuta la funcion to_le_bytes al struct HeaderMessage
        let header_message_bytes = verack_header_message.to_le_bytes();
        // THEN: se convierte a los bytes correctos segun el mensaje verack
        let expected_bytes_from_verack_header_messege: [u8; 24] = [
            11, 17, 9, 7, 118, 101, 114, 97, 99, 107, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 93, 246, 224,
            226,
        ];
        assert_eq!(
            expected_bytes_from_verack_header_messege,
            header_message_bytes
        );
    }
    #[test]
    fn header_message_of_a_version_message_marshalling_correctly_to_bytes() {
        // GIVEN: un struct HeaderMessage de un mensaje version
        let vesrion_header_message = HeaderMessage {
            start_string: [11, 17, 9, 7],
            command_name: "version".to_string(),
            payload_size: 100,
            checksum: [152, 16, 0, 0],
        };
        // WHEN: se ejecuta la funcion to_le_bytes al struct HeaderMessage
        let header_message_bytes = vesrion_header_message.to_le_bytes();
        // THEN: se convierte a los bytes correctos segun el mensaje version
        let expected_bytes_from_version_header_messege: [u8; 24] = [
            11, 17, 9, 7, 118, 101, 114, 115, 105, 111, 110, 0, 0, 0, 0, 0, 100, 0, 0, 0, 152, 16,
            0, 0,
        ];
        assert_eq!(
            expected_bytes_from_version_header_messege,
            header_message_bytes
        );
    }
}
