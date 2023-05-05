use super::message_header::*;
use super::version_payload::*;
use std::io::Error;
use std::io::{Read, Write};
use std::str::Utf8Error;
// todo: implementar tests usando mocking, simulando una conexion con un nodo y viendo si se escriben/leen correctamente los mensajes version.

#[derive(Clone, Debug)]
/// Representa un mensaje "version" segun el protocolo de bitcoin con un header (24 bytes) y un payload (variable)
pub struct VersionMessage {
    pub header: HeaderMessage,
    pub payload: VersionPayload,
}

impl VersionMessage {
    /// recibe un struct VersionMessage que representa un mensaje "version" segun protocolo de bitcoin
    /// y un stream que implemente el trait Write (en donde se pueda escribir) y escribe el mensaje serializado
    /// en bytes en el stream. Devuelve un error en caso de que no se haya podido escribir correctamente o un Ok en caso
    /// de que se haya escrito correctamente
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
    /// recibe un stream que implementa el trait Read (de donde se puede leer) y lee los bytes que corresponden al
    /// mensaje version segun el protocolo de bitcoin. Devuelve error en caso de que se no se haya podido leer correctamente
    /// del stream o en caso de que los bytes leidos no puedan ser deserializados a un struct del VersionMessage, en caso
    /// contrario, devuelve un Ok() con un VersionMessage deserializado de los bytes que leyo del stream.
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
