use crate::{compact_size_uint::CompactSizeUint, tx_in::TxIn, tx_out::TxOut};

pub struct Transaction {
    version: i32,
    txin_count: CompactSizeUint,
    tx_in: Vec<TxIn>,
    txout_count: CompactSizeUint,
    tx_out: Vec<TxOut>,
    lock_time: u32,
}

impl Transaction {
    pub fn unmarshalling(bytes: &Vec<u8>) -> Result<Transaction, &'static str> {
        // en teoria se lee el coinbase transaccion primero
        if bytes.len() < 10 {
            return Err(
                "Los bytes recibidos no corresponden a un Transaction, el largo es menor a 10 bytes",
            );
        }
        let mut offset: usize = 0;
        let mut version_bytes: [u8; 4] = [0; 4];
        version_bytes.copy_from_slice(&bytes[0..4]);
        offset += 4;
        let version = i32::from_le_bytes(version_bytes);
        let txin_count: CompactSizeUint = CompactSizeUint::unmarshalling(bytes, &mut offset);
        let amount_txin: u64 = txin_count.decoded_value();
        let tx_in: Vec<TxIn> = TxIn::unmarshalling_txins(bytes, amount_txin, &mut offset)?; // aca se actualizaria el offset tambien
        let txout_count: CompactSizeUint = CompactSizeUint::unmarshalling(bytes, &mut offset);
        let amount_txout: u64 = txout_count.decoded_value();
        let tx_out: Vec<TxOut> = TxOut::unmarshalling_txouts(bytes, amount_txout, &mut offset)?; // aca se actualizaria el offset tambien
        let mut lock_time_bytes: [u8; 4] = [0; 4];
        lock_time_bytes.copy_from_slice(&bytes[offset..(offset + 4)]);
        let lock_time = u32::from_le_bytes(lock_time_bytes);
        Ok(Transaction {
            version,
            txin_count,
            tx_in,
            txout_count,
            tx_out,
            lock_time,
        })
    }
}
#[cfg(test)]

mod test {
    use super::Transaction;
    use crate::{
        compact_size_uint::CompactSizeUint, outpoint::Outpoint, tx_in::TxIn, tx_out::TxOut,
    };

    #[test]
    fn transaction_version_is_correct() -> Result<(), &'static str> {
        let mut bytes: Vec<u8> = Vec::new();
        let version: i32 = 23;
        let version_bytes = version.to_le_bytes();
        bytes.extend_from_slice(&version_bytes[0..4]);
        let txin_count = CompactSizeUint::new(0);
        bytes.extend_from_slice(txin_count.value());
        let txout_count = CompactSizeUint::new(0);
        bytes.extend_from_slice(txout_count.value());
        let lock_time: [u8; 4] = [0; 4];
        bytes.extend_from_slice(&lock_time);
        let transaction: Transaction = Transaction::unmarshalling(&bytes)?;
        assert_eq!(version, transaction.version);
        Ok(())
    }

    #[test]
    fn transaction_version_integrate() -> Result<(), &'static str> {
        //contenedor de bytes
        let mut bytes: Vec<u8> = Vec::new();
        // version settings
        let version: i32 = 23;
        let version_bytes = version.to_le_bytes();
        bytes.extend_from_slice(&version_bytes[0..4]);
        // tx_in_count settings
        let txin_count = CompactSizeUint::new(1);
        bytes.extend_from_slice(&txin_count.marshalling()[0..1]);
        // tx_in settings
        let tx_id: [u8; 32] = [1; 32];
        let index_outpoint: u32 = 0x30000000;
        let outpoint: Outpoint = Outpoint::new(tx_id, index_outpoint);
        let compact_txin: CompactSizeUint = CompactSizeUint::new(1);
        let signature_script: Vec<u8> = vec![1];
        let sequence: u32 = 0xffffffff;
        let tx_in: TxIn = TxIn::new(outpoint, compact_txin, signature_script, sequence);
        tx_in.marshalling(&mut bytes);
        // tx_out_count settings
        let txout_count = CompactSizeUint::new(1);
        bytes.extend_from_slice(txout_count.value());
        // tx_out settings
        let value: i64 = 43;
        let pk_script_bytes: CompactSizeUint = CompactSizeUint::new(0);
        let pk_script: Vec<u8> = Vec::new();
        let tx_out = TxOut::new(value, pk_script_bytes, pk_script);
        tx_out.marshalling(&mut bytes);
        //lock_time settings
        let lock_time: [u8; 4] = [0; 4];
        bytes.extend_from_slice(&lock_time);
        let transaction: Transaction = Transaction::unmarshalling(&bytes)?;
        assert_eq!(version, transaction.version);
        assert_eq!(txin_count, transaction.txin_count);
        assert_eq!(tx_in, transaction.tx_in[0]);
        assert_eq!(txout_count, transaction.txout_count);
        assert_eq!(tx_out, transaction.tx_out[0]);
        assert_eq!(0, transaction.lock_time);
        Ok(())
    }
}
