use super::message_header::HeaderMessage;
use super::getheaders_payload::GetHeadersPayload;
use std::io::Write;

pub struct GetHeadersMessage {
    pub header: HeaderMessage,
    pub payload: GetHeadersPayload,
}


impl GetHeadersMessage {
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

