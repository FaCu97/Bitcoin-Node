use crate::{compact_size_uint::CompactSizeUint, outpoint::Outpoint};

pub struct TxIn {
    previous_output: Outpoint,
    script_bytes: CompactSizeUint,
    signature_script: Vec<u8>,
    sequence: u32,
}

impl TxIn {
    pub fn unmarshaling(bytes: &Vec<u8>) -> TxIn {
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
        for byte in 0..4 {
            sequence_bytes[byte] = bytes[byte + offset];
        }
        let sequence = u32::from_le_bytes(sequence_bytes);
        TxIn {
            previous_output,
            script_bytes,
            signature_script,
            sequence,
        }
    }

    pub fn marshaling(&self,bytes: &mut Vec<u8>){

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
        
        let expected_txin : TxIn = TxIn::unmarshaling(&bytes);
        assert_eq!(expected_txin.previous_output,outpoint);
    }

    #[test]
    fn test_unmarshaling_de_txin_serializa_correctamente_el_campo_previus_outpoint(){
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
    fn test_unmarshaling_de_txin_serializa_correctamente_el_campo_compact_size_uint(){
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
    fn test_unmarshaling_de_txin_serializa_correctamente_el_campo_signature_script(){
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
    fn test_unmarshaling_de_txin_serializa_correctamente_el_campo_sequence(){
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
}