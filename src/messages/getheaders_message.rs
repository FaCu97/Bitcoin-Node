use super::getheaders_payload::GetHeadersPayload;
use super::message_header::HeaderMessage;
use std::io::Write;
// todo: Implementar pruebas usando mocking
/// Representa un mensaje del tipo getheaders segun el protocolo de bitcoin, con su respectivo header y payload
pub struct GetHeadersMessage {
    pub header: HeaderMessage,
    pub payload: GetHeadersPayload,
}

impl GetHeadersMessage {
    /// Dado un struct GetHeadersMessage y un stream que implemente el trait Write en donde se pueda escribir,
    /// escribe el mensaje serializado a bytes en el stream y devuelve un Ok() si lo pudo escribir correctamente,
    /// y un error si no se escribio correctamente en el stream
    pub fn write_to(&self, stream: &mut dyn Write) -> std::io::Result<()> {
        let header = self.header.to_le_bytes();
        let payload: Vec<u8> = self.payload.to_le_bytes();
        let mut message: Vec<u8> = Vec::new();
        message.extend_from_slice(&header);
        message.extend(payload);
        stream.write_all(&message)?;
        stream.flush()?;
        Ok(())
    }
}
