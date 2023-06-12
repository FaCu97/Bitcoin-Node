use std::sync::{Arc, RwLock};

use crate::{account::Account, compact_size_uint::CompactSizeUint};

use super::{pubkey::Pubkey, transaction::Transaction};
#[derive(Debug, PartialEq, Clone)]
pub struct TxOut {
    value: i64,                       // Number of satoshis to spend
    pk_script_bytes: CompactSizeUint, // de 1 a 10.000 bytes
    pk_script: Pubkey, // Defines the conditions which must be satisfied to spend this output.
}

impl TxOut {
    pub fn new(value: i64, pk_script_bytes: CompactSizeUint, pk_script: Vec<u8>) -> Self {
        TxOut {
            value,
            pk_script_bytes,
            pk_script: Pubkey::new(pk_script),
        }
    }
    /// Recibe una cadena de bytes correspondiente a un TxOut
    /// Devuelve un struct TxOut
    pub fn unmarshalling(bytes: &Vec<u8>, offset: &mut usize) -> Result<TxOut, &'static str> {
        if bytes.len() - (*offset) < 9 {
            return Err(
                "Los bytes recibidos no corresponden a un TxOut, el largo es menor a 9 bytes",
            );
        }
        let mut byte_value: [u8; 8] = [0; 8];
        byte_value.copy_from_slice(&bytes[*offset..*offset + 8]);
        *offset += 8;
        let value = i64::from_le_bytes(byte_value);
        let pk_script_bytes = CompactSizeUint::unmarshalling(bytes, offset)?;
        let mut pk_script: Vec<u8> = Vec::new();
        let amount_bytes: usize = pk_script_bytes.decoded_value() as usize;
        pk_script.extend_from_slice(&bytes[*offset..(*offset + amount_bytes)]);
        *offset += amount_bytes;
        Ok(TxOut {
            value,
            pk_script_bytes,
            pk_script: Pubkey::new(pk_script),
        })
    }
    pub fn unmarshalling_txouts(
        bytes: &Vec<u8>,
        amount_txout: u64,
        offset: &mut usize,
    ) -> Result<Vec<TxOut>, &'static str> {
        let mut tx_out_list: Vec<TxOut> = Vec::new();
        let mut i = 0;
        while i < amount_txout {
            tx_out_list.push(Self::unmarshalling(bytes, offset)?);
            i += 1;
        }
        Ok(tx_out_list)
    }

    pub fn marshalling(&self, bytes: &mut Vec<u8>) {
        let value_bytes = self.value.to_le_bytes();
        bytes.extend_from_slice(&value_bytes[0..8]);
        let pk_script_bytes: Vec<u8> = self.pk_script_bytes.marshalling();
        bytes.extend_from_slice(&pk_script_bytes[0..pk_script_bytes.len()]);
        bytes.extend_from_slice(self.pk_script.bytes());
    }

    pub fn value(&self) -> i64 {
        self.value
    }

    pub fn get_adress(&self) -> Result<String, &'static str> {
        self.pk_script.generate_adress()
    }
    pub fn get_pub_key(&self) -> &Vec<u8> {
        self.pk_script.bytes()
    }
    /// Receives a list of accounts, a transaction and a pointer to the list of pending transactions
    /// and for each account in the list checks if the tx out asociate address is the same as the account
    /// address. If true, notifies the user and add the transaction to the list of pending transactions. Returns error
    /// in case the RwLock cant be accessed
    pub fn involves_user_account(
        &self,
        accounts: Vec<Account>,
        tx: Transaction,
        pending_transactions: Arc<RwLock<Vec<Transaction>>>,
    ) -> Result<(), &'static str> {
        for account in accounts {
            if !account
                .pending_transactions
                .read()
                .map_err(|_| "Error al leer puntero a vector de transacciones pendientes")?
                .contains(&tx)
            {
                let tx_asociate_address = match self.get_adress() {
                    Ok(address) => address,
                    Err(e) => e.to_string(),
                };
                if tx_asociate_address == account.address {
                    println!("%%%%%%%%%%% TRANSACCION INVOLUCRA AL USUARIO {:?}, AUN NO SE ENCUENTRA EN UN BLOQUE (PENDING) %%%%%%%%%%%%", account.address);
                    account.pending_transactions.write().map_err(|_| "Error al escribir puntero a vector de transacciones pendientes de la cuenta")?.push(tx.clone());
                    pending_transactions
                        .write()
                        .map_err(|_| "Error al leer puntero a vector de transacciones pendientes")?
                        .push(tx.clone());
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::compact_size_uint::CompactSizeUint;

    use super::TxOut;

    fn simular_flujo_de_datos(value: i64, compact_size_value: u128) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        let compact_size: CompactSizeUint = CompactSizeUint::new(compact_size_value);
        let mut pk_script: Vec<u8> = Vec::new();
        for _x in 0..compact_size_value {
            pk_script.push(1);
        }
        let tx_out: TxOut = TxOut::new(value, compact_size, pk_script);
        tx_out.marshalling(&mut bytes);
        bytes
    }

    #[test]
    fn test_unmarshalling_tx_out_invalido() {
        let bytes: Vec<u8> = vec![0; 3];
        let mut offset: usize = 0;
        let tx_out = TxOut::unmarshalling(&bytes, &mut offset);
        assert!(tx_out.is_err());
    }

    #[test]
    fn test_unmarshalling_tx_out_con_value_valido_y_0_pkscript() -> Result<(), &'static str> {
        let bytes: Vec<u8> = vec![0; 9];
        let mut offset: usize = 0;
        let tx_out = TxOut::unmarshalling(&bytes, &mut offset)?;
        assert_eq!(tx_out.value, 0);
        assert_eq!(tx_out.pk_script_bytes.decoded_value(), 0);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_tx_out_con_value_valido_y_1_pkscript() -> Result<(), &'static str> {
        let mut bytes: Vec<u8> = vec![0; 8];
        bytes[0] = 1; //Está en little endian
        let pk_script_compact_size = CompactSizeUint::new(1);
        bytes.extend_from_slice(pk_script_compact_size.value());
        let pk_script: [u8; 1] = [10; 1];
        bytes.extend_from_slice(&pk_script);
        let mut offset: usize = 0;
        let tx_out = TxOut::unmarshalling(&bytes, &mut offset)?;
        assert_eq!(tx_out.value, 1);
        assert_eq!(
            tx_out.pk_script_bytes.decoded_value(),
            pk_script_compact_size.decoded_value()
        );
        assert_eq!(tx_out.pk_script.bytes()[0], pk_script[0]);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_con_2_tx_out_devuelve_offset_esperado() -> Result<(), &'static str> {
        let bytes: Vec<u8> = vec![0; 18];
        let mut offset: usize = 0;
        let _tx_out = TxOut::unmarshalling_txouts(&bytes, 2, &mut offset)?;
        assert_eq!(offset, 18);
        Ok(())
    }
    #[test]
    fn test_unmarshalling_con_menos_bytes_de_los_esperados_devuelve_error(
    ) -> Result<(), &'static str> {
        let bytes: Vec<u8> = vec![0; 14];
        let mut offset: usize = 0;
        let tx_out: Result<Vec<TxOut>, &'static str> =
            TxOut::unmarshalling_txouts(&bytes, 2, &mut offset);
        assert!(tx_out.is_err());
        Ok(())
    }

    #[test]
    fn test_marshalling_de_tx_out_devuelve_value_esperado() -> Result<(), &'static str> {
        let expected_value: i64 = 0x302010;
        let bytes: Vec<u8> = simular_flujo_de_datos(expected_value, 3);
        let mut offset: usize = 0;
        let tx_out_expected: TxOut = TxOut::unmarshalling(&bytes, &mut offset)?;
        assert_eq!(tx_out_expected.value, expected_value);
        Ok(())
    }

    #[test]
    fn test_marshalling_de_tx_out_devuelve_pk_script_bytes_esperado() -> Result<(), &'static str> {
        let compact_size_value: u128 = 43;
        let value: i64 = 0x302010;
        let bytes: Vec<u8> = simular_flujo_de_datos(value, compact_size_value);
        let mut offset: usize = 0;
        let tx_out_expected: TxOut = TxOut::unmarshalling(&bytes, &mut offset)?;
        let compact_size_expected: CompactSizeUint = CompactSizeUint::new(compact_size_value);
        assert_eq!(tx_out_expected.pk_script_bytes, compact_size_expected);
        Ok(())
    }
    #[test]
    fn test_marshalling_de_tx_out_devuelve_pk_script_esperado() -> Result<(), &'static str> {
        let compact_size_value: u128 = 4;
        let value: i64 = 0x302010;
        let bytes: Vec<u8> = simular_flujo_de_datos(value, compact_size_value);
        let mut offset: usize = 0;
        let tx_out_expected: TxOut = TxOut::unmarshalling(&bytes, &mut offset)?;
        let pk_script_expected: Vec<u8> = vec![1, 1, 1, 1];
        assert_eq!(*tx_out_expected.pk_script.bytes(), pk_script_expected);
        Ok(())
    }
}
