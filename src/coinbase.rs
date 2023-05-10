use crate::{compact_size_uint::CompactSizeUint,outpoint::Outpoint};

pub struct Coinbase{
    previous_output: Outpoint,
    script_bytes: CompactSizeUint,
    height : Vec<u8>,
    coinbase_script: Vec<u8>,
    sequence: u32,
}

impl Coinbase{
    pub fn new(previous_output: Outpoint,script_bytes : CompactSizeUint,height : Vec<u8>,coinbase_script : Vec<u8>,sequence :u32) -> Self{
        Coinbase { previous_output,script_bytes,height,coinbase_script,sequence}
    }
    pub fn unmarshalling(bytes : &Vec<u8>,offset: &mut usize) -> Result<Coinbase,&'static str>{
        if bytes.len() - *offset < 41 {
            return Err(
                "Los bytes recibidos no corresponden a un Coinbase, el largo es menor a 41 bytes",
            );
    }
    let previous_output : Outpoint = Outpoint::unmarshalling(bytes, offset)?;
    if previous_output.is_not_a_coinbase_outpoint() {
        return Err(
            "Los bytes no corresponden a un Coinbase",
        );
    }
    let script_bytes: CompactSizeUint = CompactSizeUint::unmarshalling(bytes,offset);
    let mut height: Vec<u8> = Vec::new();
    height.extend_from_slice(&bytes[*offset..(*offset+4)]);
    *offset+=4;
    let amount_bytes_to_read : usize = script_bytes.decoded_value() as usize;
    let mut coinbase_script : Vec<u8> = Vec::new();
    coinbase_script.extend_from_slice(&bytes[*offset..(*offset+amount_bytes_to_read)]);
    *offset+=amount_bytes_to_read;
    let mut sequence_bytes: [u8; 4] = [0; 4];
    sequence_bytes.copy_from_slice(&bytes[*offset..*offset+4]);
    *offset+=4;
    let sequence = u32::from_le_bytes(sequence_bytes);
    Ok(Coinbase {
        previous_output,
        script_bytes,
        height,
        coinbase_script,
        sequence,

    })
    }

    pub fn marshalling(&self,bytes: &mut Vec<u8>){
        self.previous_output.marshalling(bytes);
        let script_bytes: Vec<u8> = self.script_bytes.marshalling();
        bytes.extend_from_slice(&script_bytes[0..script_bytes.len()]);
        bytes.extend_from_slice(&self.height[0..self.height.len()]);
        bytes.extend_from_slice(&self.coinbase_script[0..self.coinbase_script.len()]);
        let sequence_bytes: [u8; 4] = self.sequence.to_le_bytes();
        bytes.extend_from_slice(&sequence_bytes[0..4]);
    }

}

#[cfg(test)]

mod test {
    use crate::{outpoint::Outpoint, compact_size_uint::CompactSizeUint};
    use super::Coinbase;

    #[test]
    fn test_unmarshalling_coinbase_invalido() {
        let bytes: Vec<u8> = vec![0; 3];
        let mut offset :usize=0;
        let coinbase = Coinbase::unmarshalling(&bytes,&mut offset);
        assert!(coinbase.is_err());
    }

    #[test]
    fn test_unmarshalling_de_coinbase_con_outpoint_invalido_devuelve_error(){
        let mut bytes : Vec<u8> = Vec::new();
        let outpoint : Outpoint = Outpoint::new([1;32],0xffffffff);
        outpoint.marshalling(&mut bytes);
        let compact_size : CompactSizeUint = CompactSizeUint::new(1);
        bytes.extend_from_slice(&compact_size.marshalling()[0..1]);
        let height : [u8;4]  = [1;4];
        bytes.extend_from_slice(&height);
        let coinbase_script: Vec<u8> = vec![1];
        bytes.extend_from_slice(&coinbase_script[0..1]);
        let sequence: [u8;4] = [0xff;4];
        bytes.extend_from_slice(&sequence[0..4]);
        let mut offset : usize=0;
        let expected_coinbase: Result<Coinbase, &str>  = Coinbase::unmarshalling(&bytes,&mut offset);
        assert!(expected_coinbase.is_err());
    }
 
    #[test]
    fn test_unmarshalling_de_coinbase_devuelve_outpoint_esperado() -> Result<(), &'static str>{
        let mut bytes : Vec<u8> = Vec::new();
        let outpoint : Outpoint = Outpoint::new([0;32],0xffffffff);
        outpoint.marshalling(&mut bytes);
        let compact_size : CompactSizeUint = CompactSizeUint::new(1);
        bytes.extend_from_slice(&compact_size.marshalling()[0..1]);
        let height : [u8;4]  = [1;4];
        bytes.extend_from_slice(&height);
        let coinbase_script: Vec<u8> = vec![1];
        bytes.extend_from_slice(&coinbase_script[0..1]);
        let sequence: [u8;4] = [0xff;4];
        bytes.extend_from_slice(&sequence[0..4]);
        let mut offset : usize=0;
        let expected_coinbase :Coinbase = Coinbase::unmarshalling(&bytes,&mut offset)?;
        assert_eq!(expected_coinbase.previous_output,outpoint);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_de_txin_devuelve_script_bytes_esperado() -> Result<(), &'static str>{
        let mut bytes : Vec<u8> = Vec::new();
        let outpoint : Outpoint = Outpoint::new([0;32],0xffffffff);
        outpoint.marshalling(&mut bytes);
        let compact_size : CompactSizeUint = CompactSizeUint::new(1);
        bytes.extend_from_slice(&compact_size.marshalling()[0..1]);
        let height : [u8;4]  = [1;4];
        bytes.extend_from_slice(&height);
        let coinbase_script: Vec<u8> = vec![1];
        bytes.extend_from_slice(&coinbase_script[0..1]);
        let sequence: [u8;4] = [0xff;4];
        bytes.extend_from_slice(&sequence[0..4]);
        let mut offset : usize=0;
        let expected_coinbase :Coinbase = Coinbase::unmarshalling(&bytes,&mut offset)?;
        assert_eq!(expected_coinbase.script_bytes,compact_size);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_de_txin_devuelve_height_esperado() -> Result<(), &'static str>{
        let mut bytes : Vec<u8> = Vec::new();
        let outpoint : Outpoint = Outpoint::new([0;32],0xffffffff);
        outpoint.marshalling(&mut bytes);
        let compact_size : CompactSizeUint = CompactSizeUint::new(1);
        bytes.extend_from_slice(&compact_size.marshalling()[0..1]);
        let height : [u8;4]  = [1;4];
        bytes.extend_from_slice(&height);
        let coinbase_script: Vec<u8> = vec![1];
        bytes.extend_from_slice(&coinbase_script[0..1]);
        let sequence: [u8;4] = [0xff;4];
        bytes.extend_from_slice(&sequence[0..4]);
        let mut offset : usize=0;
        let expected_coinbase :Coinbase = Coinbase::unmarshalling(&bytes,&mut offset)?;
        assert_eq!(expected_coinbase.height,height);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_de_txin_devuelve_coinbase_script_esperado() -> Result<(), &'static str>{
        let mut bytes : Vec<u8> = Vec::new();
        let outpoint : Outpoint = Outpoint::new([0;32],0xffffffff);
        outpoint.marshalling(&mut bytes);
        let compact_size : CompactSizeUint = CompactSizeUint::new(1);
        bytes.extend_from_slice(&compact_size.marshalling()[0..1]);
        let height : [u8;4]  = [1;4];
        bytes.extend_from_slice(&height);
        let coinbase_script: Vec<u8> = vec![1];
        bytes.extend_from_slice(&coinbase_script[0..1]);
        let sequence: [u8;4] = [0xff;4];
        bytes.extend_from_slice(&sequence[0..4]);
        let mut offset : usize=0;
        let expected_coinbase :Coinbase = Coinbase::unmarshalling(&bytes,&mut offset)?;
        assert_eq!(expected_coinbase.coinbase_script,coinbase_script);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_de_txin_devuelve_sequence_esperado() -> Result<(), &'static str>{
        let mut bytes : Vec<u8> = Vec::new();
        let outpoint : Outpoint = Outpoint::new([0;32],0xffffffff);
        outpoint.marshalling(&mut bytes);
        let compact_size : CompactSizeUint = CompactSizeUint::new(1);
        bytes.extend_from_slice(&compact_size.marshalling()[0..1]);
        let height : [u8;4]  = [1;4];
        bytes.extend_from_slice(&height);
        let coinbase_script: Vec<u8> = vec![1];
        bytes.extend_from_slice(&coinbase_script[0..1]);
        let sequence: [u8;4] = [0xff;4];
        bytes.extend_from_slice(&sequence[0..4]);
        let mut offset : usize=0;
        let expected_coinbase :Coinbase = Coinbase::unmarshalling(&bytes,&mut offset)?;
        assert_eq!(expected_coinbase.sequence,0xffffffff);
        Ok(())
    }    
    
    #[test]
    fn test_marshalling_de_txin_serializa_correctamente_el_campo_previus_outpoint() -> Result<(), &'static str>{
        let mut bytes_coinbase:Vec<u8> = Vec::new();
        let previous_output : Outpoint = Outpoint::new([0;32],0xffffffff);
        let script_bytes : CompactSizeUint = CompactSizeUint::new(3);
        let coinbase_script : Vec<u8> = vec![0x30,0x20,0x10];
        let height : Vec<u8> = vec![0;4];
        let sequence : u32 = 0x30201000;
        let coinbase_to_marshalling : Coinbase = Coinbase { previous_output, script_bytes, height,coinbase_script, sequence};
        coinbase_to_marshalling.marshalling(&mut bytes_coinbase);
        let mut offset : usize=0;
        let coinbase_unmarshaled : Coinbase = Coinbase::unmarshalling(&bytes_coinbase,&mut offset)?;
        let expected_previous_output : Outpoint = Outpoint::new([0;32],0xffffffff);
        assert_eq!(coinbase_unmarshaled.previous_output,expected_previous_output);
        Ok(())
    }

    #[test]
    fn test_marshalling_de_txin_serializa_correctamente_el_campo_script_bytes() -> Result<(), &'static str>{
        let mut bytes_coinbase:Vec<u8> = Vec::new();
        let previous_output : Outpoint = Outpoint::new([0;32],0xffffffff);
        let script_bytes : CompactSizeUint = CompactSizeUint::new(3);
        let coinbase_script : Vec<u8> = vec![0x30,0x20,0x10];
        let height : Vec<u8> = vec![0;4];
        let sequence : u32 = 0x30201000;
        let coinbase_to_marshalling : Coinbase = Coinbase { previous_output, script_bytes, height,coinbase_script, sequence};
        coinbase_to_marshalling.marshalling(&mut bytes_coinbase);
        let mut offset : usize=0;
        let coinbase_unmarshaled : Coinbase = Coinbase::unmarshalling(&bytes_coinbase,&mut offset)?;
        let expected_script_bytes : CompactSizeUint = CompactSizeUint::new(3);
        assert_eq!(coinbase_unmarshaled.script_bytes,expected_script_bytes);
        Ok(())
    }
    /* 
    #[test]
    fn test_marshalling_de_txin_serializa_correctamente_el_campo_compact_size_uint() -> Result<(), &'static str>{
        let mut bytes_txin:Vec<u8> = Vec::new();
        let previous_output : Outpoint = Outpoint::new([1;32],0x30201000);
        let script_bytes : CompactSizeUint = CompactSizeUint::new(3);
        let signature_script : Vec<u8> = vec![0x30,0x20,0x10];
        let sequence : u32 = 0x30201000;
        let txin_to_marshalling : TxIn = TxIn { previous_output,script_bytes,signature_script,sequence};
        txin_to_marshalling.marshalling(&mut bytes_txin);
        let mut offset : usize=0;
        let txin_unmarshaled : TxIn = TxIn::unmarshalling(&bytes_txin,&mut offset)?;
        let expected_script_bytes : CompactSizeUint = CompactSizeUint::new(3);
        assert_eq!(txin_unmarshaled.script_bytes,expected_script_bytes);
        Ok(())
    }

    #[test]
    fn test_marshalling_de_txin_serializa_correctamente_el_campo_signature_script() -> Result<(), &'static str>{
        let mut bytes_txin:Vec<u8> = Vec::new();
        let previous_output : Outpoint = Outpoint::new([1;32],0x30201000);
        let script_bytes : CompactSizeUint = CompactSizeUint::new(3);
        let signature_script : Vec<u8> = vec![0x30,0x20,0x10];
        let sequence : u32 = 0x30201000;
        let txin_to_marshalling : TxIn = TxIn { previous_output,script_bytes,signature_script,sequence};
        txin_to_marshalling.marshalling(&mut bytes_txin);
        let mut offset: usize = 0;
        let txin_unmarshaled : TxIn = TxIn::unmarshalling(&bytes_txin,&mut offset)?;
        let expected_signature_script : Vec<u8> = vec![0x30,0x20,0x10];
        assert_eq!(txin_unmarshaled.signature_script,expected_signature_script);
        Ok(())
    }

    #[test]
    fn test_marshalling_de_txin_serializa_correctamente_el_campo_sequence() -> Result<(), &'static str>{
        let mut bytes_txin:Vec<u8> = Vec::new();
        let previous_output : Outpoint = Outpoint::new([1;32],0x30201000);
        let script_bytes : CompactSizeUint = CompactSizeUint::new(3);
        let signature_script : Vec<u8> = vec![0x30,0x20,0x10];
        let sequence : u32 = 0x30201000;
        let txin_to_marshalling : TxIn = TxIn { previous_output,script_bytes,signature_script,sequence};
        let mut offset : usize = 0;
        txin_to_marshalling.marshalling(&mut bytes_txin);
        let txin_unmarshaled : TxIn = TxIn::unmarshalling(&bytes_txin,& mut offset)?;
        assert_eq!(txin_unmarshaled.sequence,sequence);
        Ok(())
    }*/
}