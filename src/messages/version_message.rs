use super::message_header::*;
use super::version_payload::*;
use std::io::Error;
use std::io::{Read, Write};
use std::str::Utf8Error;
// todo: implementar tests usando mocking, simulando una conexion con un nodo y viendo si se escriben/leen correctamente los mensajes version.

#[derive(Clone, Debug)]
pub struct VersionMessage {
    pub header: HeaderMessage,
    pub payload: VersionPayload,
}

impl VersionMessage {
    pub fn write_to(&self, stream: &mut dyn Write) -> std::io::Result<()> {
        let header = self.header.to_le_bytes();
        let payload = self.payload.to_le_bytes();
        let mut message: Vec<u8> = Vec::new();
        message.extend_from_slice(&header);
        message.extend(payload);
        stream.write_all(&message)?;
        stream.flush()?;
        Ok(())
    }

    pub fn read_from(stream: &mut dyn Read) -> Result<VersionMessage, Error> {
        let mut buffer_num = [0; 24];
        stream.read_exact(&mut buffer_num)?;
        let header = HeaderMessage::from_le_bytes(buffer_num).map_err(|err: Utf8Error| {
            Error::new(std::io::ErrorKind::InvalidData, err.to_string())
        })?;
        let payload_large = header.payload_size;
        let mut buffer_num = vec![0; payload_large as usize];
        stream.read_exact(&mut buffer_num)?;
        let payload = VersionPayload::from_le_bytes(&buffer_num).map_err(|err: Utf8Error| {
            Error::new(std::io::ErrorKind::InvalidData, err.to_string())
        })?;
        Ok(VersionMessage { header, payload })
    }
}
