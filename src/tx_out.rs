use crate::compact_size_uint::CompactSizeUint;

pub struct TxOut {
    value: i64,                       // Number of satoshis to spend
    pk_script_bytes: CompactSizeUint, // de 1 a 10.000 bytes
    pk_script: Vec<u8>, // Defines the conditions which must be satisfied to spend this output.
}

impl TxOut {
    /// Recibe una cadena de bytes correspondiente a un TxOut
    /// Devuelve un struct TxOut
    pub fn unmarshalling(bytes: &Vec<u8>) -> Result<TxOut, &'static str> {
        if bytes.len() < 9 {
            return Err(
                "Los bytes recibidos no corresponden a un TxOut, el largo es menor a 9 bytes",
            );
        }

        let mut offset = 0;
        let mut byte_value: [u8; 8] = [0; 8];
        byte_value.copy_from_slice(&bytes[0..8]);
        offset += 8;
        let value = i64::from_le_bytes(byte_value);
        let pk_script_bytes = CompactSizeUint::unmarshaling(bytes, &mut offset);
        let mut pk_script: Vec<u8> = Vec::new();
        pk_script.extend_from_slice(&bytes[offset..bytes.len()]);

        Ok(TxOut {
            value,
            pk_script_bytes,
            pk_script,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{compact_size_uint::CompactSizeUint, tx_out::TxOut};
    #[test]
    fn test_unmarshaling_tx_out_invalido() {
        let bytes: Vec<u8> = vec![0; 3];
        let tx_out = TxOut::unmarshalling(&bytes);
        assert!(tx_out.is_err());
    }

    #[test]
    fn test_unmarshaling_tx_out_con_value_valido_y_0_pkscript() -> Result<(), &'static str> {
        let bytes: Vec<u8> = vec![0; 9];
        let tx_out = TxOut::unmarshalling(&bytes)?;
        assert_eq!(tx_out.value, 0);
        assert_eq!(tx_out.pk_script_bytes.decoded_value(), 0);
        Ok(())
    }

    #[test]
    fn test_unmarshaling_tx_out_con_value_valido_y_1_pkscript() -> Result<(), &'static str> {
        let mut bytes: Vec<u8> = vec![0; 8];
        bytes[0] = 1; //Está en little endian
        let pk_script_compact_size = CompactSizeUint::new(1);
        bytes.extend_from_slice(pk_script_compact_size.value());
        let pk_script: [u8; 1] = [10; 1];
        bytes.extend_from_slice(&pk_script);
        let tx_out = TxOut::unmarshalling(&bytes)?;
        assert_eq!(tx_out.value, 1);
        assert_eq!(
            tx_out.pk_script_bytes.decoded_value(),
            pk_script_compact_size.decoded_value()
        );
        assert_eq!(tx_out.pk_script[0], pk_script[0]);
        Ok(())
    }
}
