use crate::{compact_size_uint::CompactSizeUint, outpoint::Outpoint};

pub struct TxIn {
    pub previous_output: Outpoint,
    pub script_bytes: CompactSizeUint,
    pub signature_script: Vec<u8>,
    pub sequence: u32,
}

impl TxIn {
    pub fn unmarshaling(bytes: &[u8]) -> TxIn {
        let mut offset: usize = 0;
        let previous_output: Outpoint = Outpoint::unmarshaling(bytes);
        offset += 36;
        let script_bytes: CompactSizeUint = CompactSizeUint::unmarshaling(bytes, &mut offset);
        let mut signature_script: Vec<u8> = Vec::new();
        let amount_bytes_to_read : u64 = script_bytes.decoded_value();
        for byte in 0..amount_bytes_to_read{
        signature_script.push(bytes[offset+(byte as usize)]);
        }
        offset+=amount_bytes_to_read as usize;
        let mut sequence_bytes: [u8; 4] = [0; 4];
        sequence_bytes[..4].copy_from_slice(&bytes[offset..(4 + offset)]);
        let sequence = u32::from_le_bytes(sequence_bytes);
        TxIn {
            previous_output,
            script_bytes,
            signature_script,
            sequence,
        }
    }

    pub fn marshaling(&self, bytes: &mut [u8]){

    }
}
#[cfg(test)]

mod test {
    use crate::{outpoint::Outpoint, compact_size_uint::CompactSizeUint};
    use super::TxIn;
 
    #[test]
    fn test_unmarshaling_de_txid_devuelve_outpoint_esperado(){
        let mut bytes : Vec<u8> = Vec::new();
        let outpoint : Outpoint = Outpoint::new([1;32],0x30201000);
        outpoint.marshaling(&mut bytes);
        let compact_size : CompactSizeUint = CompactSizeUint::new(1);
        bytes.extend_from_slice(&compact_size.marshalling()[0..1]);
        let signature_script: Vec<u8> = vec![1];
        bytes.extend_from_slice(&signature_script[0..1]);
        let sequence: [u8;4] = [0xff;4];
        bytes.extend_from_slice(&sequence[0..4]);
        let expected_txin : TxIn = TxIn::unmarshaling(&bytes);
        assert_eq!(expected_txin.previous_output,outpoint);
    }
    #[test]
    fn test_unmarshaling_de_txid_devuelve_script_bytes_esperado(){
        let mut bytes : Vec<u8> = Vec::new();
        let outpoint : Outpoint = Outpoint::new([1;32],0x30201000);
        outpoint.marshaling(&mut bytes);
        let compact_size : CompactSizeUint = CompactSizeUint::new(1);
        bytes.extend_from_slice(&compact_size.marshalling()[0..1]);
        let signature_script: Vec<u8> = vec![1];
        bytes.extend_from_slice(&signature_script[0..1]);
        let sequence: [u8;4] = [0xff;4];
        bytes.extend_from_slice(&sequence[0..4]);
        let expected_txin : TxIn = TxIn::unmarshaling(&bytes);
        assert_eq!(expected_txin.script_bytes,compact_size);
    }

    #[test]
    fn test_unmarshaling_de_txid_devuelve_signature_script_esperado(){
        let mut bytes : Vec<u8> = Vec::new();
        let outpoint : Outpoint = Outpoint::new([1;32],0x30201000);
        outpoint.marshaling(&mut bytes);
        let compact_size : CompactSizeUint = CompactSizeUint::new(1);
        bytes.extend_from_slice(&compact_size.marshalling()[0..1]);
        let signature_script: Vec<u8> = vec![1];
        bytes.extend_from_slice(&signature_script[0..1]);
        let sequence: [u8;4] = [0xff;4];
        bytes.extend_from_slice(&sequence[0..4]);
        let expected_txin : TxIn = TxIn::unmarshaling(&bytes);
        assert_eq!(expected_txin.signature_script,signature_script);
    }

    #[test]
    fn test_unmarshaling_de_txid_devuelve_sequence_esperado(){
        let mut bytes : Vec<u8> = Vec::new();
        let outpoint : Outpoint = Outpoint::new([1;32],0x30201000);
        outpoint.marshaling(&mut bytes);
        let compact_size : CompactSizeUint = CompactSizeUint::new(1);
        bytes.extend_from_slice(&compact_size.marshalling()[0..1]);
        let signature_script: Vec<u8> = vec![1];
        bytes.extend_from_slice(&signature_script[0..1]);
        let sequence: [u8;4] = [0xff;4];
        bytes.extend_from_slice(&sequence[0..4]);
        let expected_txin : TxIn = TxIn::unmarshaling(&bytes);
        assert_eq!(expected_txin.sequence,0xffffffff);
    }
/* 
    #[test]
    fn test_marshaling_de_txin_serializa_correctamente_el_campo_previus_outpoint(){
        let mut bytes_txin:Vec<u8> = Vec::new();
        let previous_output : Outpoint = Outpoint::new([1;32],0x30201000);
        let script_bytes : CompactSizeUint = CompactSizeUint::new(3);
        let signature_script : Vec<u8> = vec![0x30,0x20,0x10];
        let sequence : u32 = 0x30201000;
        let txin_to_marshaling : TxIn = TxIn { previous_output,script_bytes,signature_script,sequence};
        txin_to_marshaling.marshaling(&mut bytes_txin);
        let txin_unmarshaled : TxIn = TxIn::unmarshaling(&bytes_txin);
        let expected_previous_output : Outpoint = Outpoint::new([1;32],0x30201000);
        assert_eq!(txin_unmarshaled.previous_output,expected_previous_output);
    }

    #[test]
    fn test_marshaling_de_txin_serializa_correctamente_el_campo_compact_size_uint(){
        let mut bytes_txin:Vec<u8> = Vec::new();
        let previous_output : Outpoint = Outpoint::new([1;32],0x30201000);
        let script_bytes : CompactSizeUint = CompactSizeUint::new(3);
        let signature_script : Vec<u8> = vec![0x30,0x20,0x10];
        let sequence : u32 = 0x30201000;
        let txin_to_marshaling : TxIn = TxIn { previous_output,script_bytes,signature_script,sequence};
        txin_to_marshaling.marshaling(&mut bytes_txin);
        let txin_unmarshaled : TxIn = TxIn::unmarshaling(&bytes_txin);
        let expected_script_bytes : CompactSizeUint = CompactSizeUint::new(3);
        assert_eq!(txin_unmarshaled.script_bytes,expected_script_bytes);
    }

    #[test]
    fn test_marshaling_de_txin_serializa_correctamente_el_campo_signature_script(){
        let mut bytes_txin:Vec<u8> = Vec::new();
        let previous_output : Outpoint = Outpoint::new([1;32],0x30201000);
        let script_bytes : CompactSizeUint = CompactSizeUint::new(3);
        let signature_script : Vec<u8> = vec![0x30,0x20,0x10];
        let sequence : u32 = 0x30201000;
        let txin_to_marshaling : TxIn = TxIn { previous_output,script_bytes,signature_script,sequence};
        txin_to_marshaling.marshaling(&mut bytes_txin);
        let txin_unmarshaled : TxIn = TxIn::unmarshaling(&bytes_txin);
        let expected_signature_script : Vec<u8> = vec![0x30,0x20,0x10];
        assert_eq!(txin_unmarshaled.signature_script,expected_signature_script);
    }

    #[test]
    fn test_marshaling_de_txin_serializa_correctamente_el_campo_sequence(){
        let mut bytes_txin:Vec<u8> = Vec::new();
        let previous_output : Outpoint = Outpoint::new([1;32],0x30201000);
        let script_bytes : CompactSizeUint = CompactSizeUint::new(3);
        let signature_script : Vec<u8> = vec![0x30,0x20,0x10];
        let sequence : u32 = 0x30201000;
        let txin_to_marshaling : TxIn = TxIn { previous_output,script_bytes,signature_script,sequence};
        txin_to_marshaling.marshaling(&mut bytes_txin);
        let txin_unmarshaled : TxIn = TxIn::unmarshaling(&bytes_txin);
        assert_eq!(txin_unmarshaled.sequence,sequence);
    }
    */
}