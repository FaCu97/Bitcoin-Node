use super::message_header::HeaderMessage;
use std::io::Error;
use std::io::{Read, Write};
use std::str::Utf8Error;
// todo: implementar tests usando mocking, simulando una conexion con un nodo y viendo si se escriben/leen correctamente los mensajes sin payload.
#[derive(Clone, Debug)]
/// Representa cualquier mensaje que solo tenga header y no necesite payload segun el protocolo bitcoin
pub struct NonePayloadMessage {
    pub header: HeaderMessage,
}
impl NonePayloadMessage {
    /// recibe un struct NonePayloadMessage que representa un mensaje sin payload segun protocolo de bitcoin
    /// y un stream que implemente el trait Write (en donde se pueda escribir) y escribe el mensaje serializado
    /// en bytes en el stream. Devuelve un error en caso de que no se haya podido escribir correctamente o un Ok en caso
    /// de que se haya escrito correctamente
    pub fn write_to(&self, stream: &mut dyn Write) -> std::io::Result<()> {
        let header = self.header.to_le_bytes();
        stream.write_all(&header)?;
        stream.flush()?;
        Ok(())
    }
    /// recibe un stream que implementa el trait Read (de donde se puede leer) y lee los bytes que corresponden a un
    /// mensaje sin payload segun el protocolo de bitcoin. Devuelve error en caso de que se no se haya podido leer correctamente
    /// del stream o en caso de que los bytes leidos no puedan ser deserializados a un struct de tipo NonePayloadMessage, en caso
    /// contrario, devuelve un Ok() con un NonePayloadMessage deserializado de los bytes que leyo del stream.
    pub fn read_from(stream: &mut dyn Read) -> Result<SendheadersMessage, Error> {
        let mut buffer_num = [0; 24];
        stream.read_exact(&mut buffer_num)?;
        let header = HeaderMessage::from_le_bytes(buffer_num).map_err(|err: Utf8Error| {
            Error::new(std::io::ErrorKind::InvalidData, err.to_string())
        })?;
        Ok(SendheadersMessage { header })
    }
}
