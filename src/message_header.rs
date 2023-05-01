use std::str::Utf8Error;

#[derive(Clone, Debug)]
pub struct HeaderMessage {
    start_string: [u8; 4],
    command_name: String,
    payload_size: u32,
    checksum: [u8; 4],
}


impl HeaderMessage {
    pub fn to_le_bytes(&self) -> [u8; 24] {
        let mut header_message_bytes: [u8; 24] = [0; 24];
        header_message_bytes[0..4].copy_from_slice(&self.start_string);
        header_message_bytes[4..16].copy_from_slice(&command_name_to_bytes(&self.command_name));
        header_message_bytes[16..20].copy_from_slice(&self.payload_size.to_le_bytes());
        header_message_bytes[20..24].copy_from_slice(&self.checksum);
        header_message_bytes
    }
    pub fn from_le_bytes(bytes: [u8; 24]) -> Result<Self, Utf8Error> {
        let mut start_string = [0; 4];
        let mut counter = 0;
        for i in 0..4{
            start_string[i] = bytes[i];
        }
        counter += 4;
        let mut command_name_bytes = [0; 12];
        for i in 0..12 {
            command_name_bytes[i] = bytes[i + counter];
        }
        counter += 12;
        let command_name = std::str::from_utf8(&command_name_bytes)?.to_string();
        let mut payload_size_bytes: [u8; 4] = [0; 4];
        for i in 0..4 {
            payload_size_bytes[i] = bytes[i + counter];
        }
        counter += 4;
        let payload_size = u32::from_le_bytes(payload_size_bytes);
        let mut checksum = [0; 4];
        for i in 0..4{
            checksum[i] = bytes[i + counter];
        }
        Ok(HeaderMessage{
            start_string,
            command_name,
            payload_size,
            checksum,
        })
    }
}


/// Recibe un String que representa el nombre del comando del Header Message
/// y devuelve los bytes que representan ese string (ASCII) seguido de 0x00 para
/// completar los 12 bytes
/// little-endian
fn command_name_to_bytes(command: &String) -> [u8; 12] {
    let mut command_name_bytes = [0; 12];
    let command_bytes = command.as_bytes();
    command_name_bytes[..command_bytes.len()]
        .copy_from_slice(&command_bytes[..command_bytes.len()]);
    command_name_bytes
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn header_message_bytes_from_verack_message_unmarshalling_correctly() {
        // GIVEN : un header messege del mensaje verack en bytes
        let header_message_bytes: [u8; 24] = [
            11, 17, 9, 7,
            118, 101, 114, 97, 99, 107, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
            93, 246, 224, 226
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
            11, 17, 9, 7,
            118, 101, 114, 115, 105, 111, 110, 0, 0, 0, 0, 0,
            100, 0, 0, 0,
            152, 16, 0, 0
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
            11, 17, 9, 7,
            12, 101, 114, 13, 240, 111, 110, 1, 0, 0, 0, 11,
            100, 0, 0, 0,
            152, 16, 0, 0
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
        let verack_header_message = HeaderMessage{
            start_string: [11, 17, 9, 7],
            command_name: "verack".to_string(),
            payload_size: 0,
            checksum: [93, 246, 224, 226],
        };
        // WHEN: se ejecuta la funcion to_le_bytes al struct HeaderMessage
        let header_message_bytes = verack_header_message.to_le_bytes();
        // THEN: se convierte a los bytes correctos segun el mensaje verack
        let expected_bytes_from_verack_header_messege: [u8; 24] = [
            11, 17, 9, 7,
            118, 101, 114, 97, 99, 107, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
            93, 246, 224, 226
        ];
        assert_eq!(expected_bytes_from_verack_header_messege, header_message_bytes);
    }
    #[test]
    fn header_message_of_a_version_message_marshalling_correctly_to_bytes() {
        // GIVEN: un struct HeaderMessage de un mensaje version
        let vesrion_header_message = HeaderMessage{
            start_string: [11, 17, 9, 7],
            command_name: "version".to_string(),
            payload_size: 100,
            checksum: [152, 16, 0, 0],
        };
        // WHEN: se ejecuta la funcion to_le_bytes al struct HeaderMessage
        let header_message_bytes = vesrion_header_message.to_le_bytes();
        // THEN: se convierte a los bytes correctos segun el mensaje version
        let expected_bytes_from_version_header_messege: [u8; 24] = [
            11, 17, 9, 7,
            118, 101, 114, 115, 105, 111, 110, 0, 0, 0, 0, 0,
            100, 0, 0, 0,
            152, 16, 0, 0
        ];
        assert_eq!(expected_bytes_from_version_header_messege, header_message_bytes);
    }

    
}