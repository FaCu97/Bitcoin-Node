use crate::compact_size_uint::CompactSizeUint;

use super::outpoint::Outpoint;
#[derive(Debug, PartialEq, Clone)]
pub struct TxIn {
    previous_output: Outpoint,
    script_bytes: CompactSizeUint,
    pub height: Option<Vec<u8>>,
    pub signature_script: Vec<u8>,
    sequence: u32,
}

impl TxIn {
    pub fn new(
        previous_output: Outpoint,
        script_bytes: CompactSizeUint,
        height: Option<Vec<u8>>,
        signature_script: Vec<u8>,
        sequence: u32,
    ) -> Self {
        TxIn {
            previous_output,
            script_bytes,
            height,
            signature_script,
            sequence,
        }
    }
    /// recibe un vector de byes que contiene un txin y un offset indicando la posicion donde empieza el txin
    /// devuelve un txin completando los campos con lo que esta en los bytes en caso de que todo este bien
    /// y un string indicando el error cuando algo falla. tambien actualiza el offset
    pub fn unmarshalling(bytes: &Vec<u8>, offset: &mut usize) -> Result<TxIn, &'static str> {
        if bytes.len() - *offset < 41 {
            return Err(
                "Los bytes recibidos no corresponden a un TxIn, el largo es menor a 41 bytes",
            );
        }
        let previous_output: Outpoint = Outpoint::unmarshalling(bytes, offset)?;
        let script_bytes: CompactSizeUint = CompactSizeUint::unmarshalling(bytes, offset)?;
        let mut height: Option<Vec<u8>> = None;
        let mut bytes_for_height = 0;
        if previous_output.is_a_coinbase_outpoint() {
            if script_bytes.decoded_value() > 100 {
                return Err(
                    "Los bytes recibidos no corresponden a un coinbase TxIn, el largo del script es mayor a 100 bytes",
                );
            }
            let mut height_bytes: Vec<u8> = Vec::new();
            height_bytes.extend_from_slice(&bytes[*offset..(*offset + 4)]);
            height = Some(height_bytes);
            *offset += 4;
            bytes_for_height = 4;
        }
        let mut signature_script: Vec<u8> = Vec::new();
        let amount_bytes_to_read: usize = script_bytes.decoded_value() as usize;
        signature_script.extend_from_slice(
            &bytes[*offset..(*offset + amount_bytes_to_read - bytes_for_height)],
        );
        *offset += amount_bytes_to_read - bytes_for_height;
        let mut sequence_bytes: [u8; 4] = [0; 4];
        sequence_bytes.copy_from_slice(&bytes[*offset..*offset + 4]);
        *offset += 4;
        let sequence = u32::from_le_bytes(sequence_bytes);
        Ok(TxIn {
            previous_output,
            script_bytes,
            height,
            signature_script,
            sequence,
        })
    }

    pub fn unmarshalling_txins(
        bytes: &Vec<u8>,
        amount_txin: u64,
        offset: &mut usize,
    ) -> Result<Vec<TxIn>, &'static str> {
        let mut tx_in_list: Vec<TxIn> = Vec::new();
        let mut i = 0;
        while i < amount_txin {
            tx_in_list.push(Self::unmarshalling(bytes, offset)?);
            i += 1;
        }
        Ok(tx_in_list)
    }

    pub fn marshalling(&self, bytes: &mut Vec<u8>) {
        self.previous_output.marshalling(bytes);
        let script_bytes: Vec<u8> = self.script_bytes.marshalling();
        bytes.extend_from_slice(&script_bytes);
        if self.is_coinbase() {
            if let Some(height) = &self.height {
                bytes.extend_from_slice(height)
            }
        }
        bytes.extend_from_slice(&self.signature_script);
        let sequence_bytes: [u8; 4] = self.sequence.to_le_bytes();
        bytes.extend_from_slice(&sequence_bytes);
    }

    pub fn is_coinbase(&self) -> bool {
        self.height.is_some()
    }

    pub fn outpoint(&self) -> Outpoint {
        self.previous_output
    }
}
#[cfg(test)]

mod test {
    use super::TxIn;
    use crate::{compact_size_uint::CompactSizeUint, transactions::outpoint::Outpoint};

    fn simular_flujo_de_datos(
        tx_id: [u8; 32],
        index: u32,
        compact_size_value: u128,
        height: Option<Vec<u8>>,
        sequence: u32,
    ) -> Vec<u8> {
        let mut bytes_txin: Vec<u8> = Vec::new();
        let previous_output: Outpoint = Outpoint::new(tx_id, index);
        let script_bytes: CompactSizeUint = CompactSizeUint::new(compact_size_value);
        let mut signature_script: Vec<u8> = Vec::new();
        for _x in 0..compact_size_value {
            signature_script.push(1);
        }
        let txin_to_marshalling: TxIn = TxIn {
            previous_output,
            script_bytes,
            height,
            signature_script,
            sequence,
        };
        txin_to_marshalling.marshalling(&mut bytes_txin);
        bytes_txin
    }

    #[test]
    fn test_unmarshalling_tx_in_invalido() {
        let bytes: Vec<u8> = vec![0; 3];
        let mut offset: usize = 0;
        let tx_in = TxIn::unmarshalling(&bytes, &mut offset);
        assert!(tx_in.is_err());
    }

    #[test]
    fn test_unmarshalling_de_txin_devuelve_outpoint_esperado() -> Result<(), &'static str> {
        let mut bytes: Vec<u8> = Vec::new();
        let outpoint: Outpoint = Outpoint::new([1; 32], 0x30201000);
        outpoint.marshalling(&mut bytes);
        let compact_size: CompactSizeUint = CompactSizeUint::new(1);
        bytes.extend_from_slice(&compact_size.marshalling()[0..1]);
        let signature_script: Vec<u8> = vec![1];
        bytes.extend_from_slice(&signature_script[0..1]);
        let sequence: [u8; 4] = [0xff; 4];
        bytes.extend_from_slice(&sequence[0..4]);
        let mut offset: usize = 0;
        let expected_txin: TxIn = TxIn::unmarshalling(&bytes, &mut offset)?;
        assert_eq!(expected_txin.previous_output, outpoint);
        Ok(())
    }
    #[test]
    fn test_unmarshalling_de_txin_devuelve_script_bytes_esperado() -> Result<(), &'static str> {
        let mut bytes: Vec<u8> = Vec::new();
        let outpoint: Outpoint = Outpoint::new([1; 32], 0x30201000);
        outpoint.marshalling(&mut bytes);
        let compact_size: CompactSizeUint = CompactSizeUint::new(1);
        bytes.extend_from_slice(&compact_size.marshalling()[0..1]);
        let signature_script: Vec<u8> = vec![1];
        bytes.extend_from_slice(&signature_script[0..1]);
        let sequence: [u8; 4] = [0xff; 4];
        bytes.extend_from_slice(&sequence[0..4]);
        let mut offset: usize = 0;
        let expected_txin: TxIn = TxIn::unmarshalling(&bytes, &mut offset)?;
        assert_eq!(expected_txin.script_bytes, compact_size);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_de_txin_devuelve_signature_script_esperado() -> Result<(), &'static str> {
        let mut bytes: Vec<u8> = Vec::new();
        let outpoint: Outpoint = Outpoint::new([1; 32], 0x30201000);
        outpoint.marshalling(&mut bytes);
        let compact_size: CompactSizeUint = CompactSizeUint::new(1);
        bytes.extend_from_slice(&compact_size.marshalling()[0..1]);
        let signature_script: Vec<u8> = vec![1];
        bytes.extend_from_slice(&signature_script[0..1]);
        let sequence: [u8; 4] = [0xff; 4];
        bytes.extend_from_slice(&sequence[0..4]);
        let mut offset: usize = 0;
        let expected_txin: TxIn = TxIn::unmarshalling(&bytes, &mut offset)?;
        assert_eq!(expected_txin.signature_script, signature_script);
        assert_eq!(offset, 42);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_de_txin_devuelve_sequence_esperado() -> Result<(), &'static str> {
        let mut bytes: Vec<u8> = Vec::new();
        let outpoint: Outpoint = Outpoint::new([1; 32], 0x30201000);
        outpoint.marshalling(&mut bytes);
        let compact_size: CompactSizeUint = CompactSizeUint::new(1);
        bytes.extend_from_slice(&compact_size.marshalling()[0..1]);
        let signature_script: Vec<u8> = vec![1];
        bytes.extend_from_slice(&signature_script[0..1]);
        let sequence: [u8; 4] = [0xff; 4];
        bytes.extend_from_slice(&sequence[0..4]);
        let mut offset: usize = 0;
        let expected_txin: TxIn = TxIn::unmarshalling(&bytes, &mut offset)?;
        assert_eq!(expected_txin.sequence, 0xffffffff);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_de_txin_devuelve_offset_esperado() -> Result<(), &'static str> {
        let mut bytes: Vec<u8> = Vec::new();
        let outpoint: Outpoint = Outpoint::new([1; 32], 0x30201000);
        outpoint.marshalling(&mut bytes);
        let compact_size: CompactSizeUint = CompactSizeUint::new(1);
        bytes.extend_from_slice(&compact_size.marshalling()[0..1]);
        let signature_script: Vec<u8> = vec![1];
        bytes.extend_from_slice(&signature_script[0..1]);
        let sequence: [u8; 4] = [0xff; 4];
        bytes.extend_from_slice(&sequence[0..4]);
        let mut offset: usize = 0;
        let _expected_txin: TxIn = TxIn::unmarshalling(&bytes, &mut offset)?;
        assert_eq!(offset, 42);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_de_2_txin_devuelve_offset_esperado() -> Result<(), &'static str> {
        let mut bytes: Vec<u8> = Vec::new();
        let outpoint_1: Outpoint = Outpoint::new([1; 32], 0x30201000);
        outpoint_1.marshalling(&mut bytes);
        let compact_size_1: CompactSizeUint = CompactSizeUint::new(1);
        bytes.extend_from_slice(&compact_size_1.marshalling()[0..1]);
        let signature_script_1: Vec<u8> = vec![1];
        bytes.extend_from_slice(&signature_script_1[0..1]);
        let sequence_1: [u8; 4] = [0xff; 4];
        bytes.extend_from_slice(&sequence_1[0..4]);
        let outpoint_2: Outpoint = Outpoint::new([1; 32], 0x30201000);
        outpoint_2.marshalling(&mut bytes);
        let compact_size_2: CompactSizeUint = CompactSizeUint::new(1);
        bytes.extend_from_slice(&compact_size_2.marshalling()[0..1]);
        let signature_script_2: Vec<u8> = vec![1];
        bytes.extend_from_slice(&signature_script_2[0..1]);
        let sequence_2: [u8; 4] = [0xff; 4];
        bytes.extend_from_slice(&sequence_2[0..4]);
        let mut offset: usize = 0;
        let _expected_txin: Vec<TxIn> = TxIn::unmarshalling_txins(&bytes, 2, &mut offset)?;
        assert_eq!(offset, 84);
        Ok(())
    }

    #[test]
    fn test_marshalling_de_txin_serializa_correctamente_el_campo_previus_outpoint(
    ) -> Result<(), &'static str> {
        let tx_id: [u8; 32] = [1; 32];
        let index: u32 = 0x30201000;
        let bytes_txin: Vec<u8> = simular_flujo_de_datos(tx_id, index, 2, None, 0xffffffff);
        let mut offset: usize = 0;
        let txin_unmarshaled: TxIn = TxIn::unmarshalling(&bytes_txin, &mut offset)?;
        let expected_previous_output: Outpoint = Outpoint::new(tx_id, index);
        assert_eq!(txin_unmarshaled.previous_output, expected_previous_output);
        Ok(())
    }

    #[test]
    fn test_marshalling_de_txin_serializa_correctamente_el_campo_compact_size_uint(
    ) -> Result<(), &'static str> {
        let tx_id: [u8; 32] = [1; 32];
        let index: u32 = 0x30201000;
        let bytes_txin: Vec<u8> = simular_flujo_de_datos(tx_id, index, 2, None, 0xffffffff);
        let mut offset: usize = 0;
        let txin_unmarshaled: TxIn = TxIn::unmarshalling(&bytes_txin, &mut offset)?;
        let expected_script_bytes: CompactSizeUint = CompactSizeUint::new(2);
        assert_eq!(txin_unmarshaled.script_bytes, expected_script_bytes);
        Ok(())
    }

    #[test]
    fn test_marshalling_de_txin_serializa_correctamente_el_campo_signature_script(
    ) -> Result<(), &'static str> {
        let tx_id: [u8; 32] = [1; 32];
        let index: u32 = 0x30201000;
        let bytes_txin: Vec<u8> = simular_flujo_de_datos(tx_id, index, 2, None, 0xffffffff);
        let mut offset: usize = 0;
        let txin_unmarshaled: TxIn = TxIn::unmarshalling(&bytes_txin, &mut offset)?;
        let expected_signature_script: Vec<u8> = vec![1, 1];
        assert_eq!(txin_unmarshaled.signature_script, expected_signature_script);
        Ok(())
    }

    #[test]
    fn test_marshalling_de_txin_serializa_correctamente_el_campo_sequence(
    ) -> Result<(), &'static str> {
        let tx_id: [u8; 32] = [1; 32];
        let index: u32 = 0x30201000;
        let sequence: u32 = 0x302010;
        let bytes_txin: Vec<u8> = simular_flujo_de_datos(tx_id, index, 2, None, sequence);
        let mut offset: usize = 0;
        let txin_unmarshaled: TxIn = TxIn::unmarshalling(&bytes_txin, &mut offset)?;
        assert_eq!(txin_unmarshaled.sequence, sequence);
        Ok(())
    }
}
