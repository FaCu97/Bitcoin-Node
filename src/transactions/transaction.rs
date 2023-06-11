use std::error::Error;

use bitcoin_hashes::{sha256d, Hash};

use crate::{account::Account, compact_size_uint::CompactSizeUint, utxo_tuple::UtxoTuple};

use super::{tx_in::TxIn, tx_out::TxOut};

/// Guarda el txid(hash de la transaccion) y el vector con los utxos (valor e indice)
#[derive(Debug, PartialEq, Clone)]
pub struct Transaction {
    pub version: i32,
    pub txin_count: CompactSizeUint,
    pub tx_in: Vec<TxIn>,
    pub txout_count: CompactSizeUint,
    pub tx_out: Vec<TxOut>,
    pub lock_time: u32,
}

impl Transaction {
    pub fn new(
        version: i32,
        txin_count: CompactSizeUint,
        tx_in: Vec<TxIn>,
        txout_count: CompactSizeUint,
        tx_out: Vec<TxOut>,
        lock_time: u32,
    ) -> Self {
        Transaction {
            version,
            txin_count,
            tx_in,
            txout_count,
            tx_out,
            lock_time,
        }
    }

    pub fn unmarshalling(bytes: &Vec<u8>, offset: &mut usize) -> Result<Transaction, &'static str> {
        // en teoria se lee el coinbase transaccion primero
        if bytes.len() < 10 {
            return Err(
                "Los bytes recibidos no corresponden a un Transaction, el largo es menor a 10 bytes",
            );
        }
        let mut version_bytes: [u8; 4] = [0; 4];
        version_bytes.copy_from_slice(&bytes[*offset..(*offset + 4)]);
        *offset += 4;
        let version = i32::from_le_bytes(version_bytes);
        let txin_count: CompactSizeUint = CompactSizeUint::unmarshalling(bytes, &mut *offset)?;
        let amount_txin: u64 = txin_count.decoded_value();
        let tx_in: Vec<TxIn> = TxIn::unmarshalling_txins(bytes, amount_txin, &mut *offset)?; // aca se actualizaria el *offset tambien
        if tx_in[0].is_coinbase() && txin_count.decoded_value() != 1 {
            return Err("una coinbase transaction no puede tener mas de un input");
        }
        let txout_count: CompactSizeUint = CompactSizeUint::unmarshalling(bytes, &mut *offset)?;
        let amount_txout: u64 = txout_count.decoded_value();
        let tx_out: Vec<TxOut> = TxOut::unmarshalling_txouts(bytes, amount_txout, &mut *offset)?; // aca se actualizaria el *offset tambien
        let mut lock_time_bytes: [u8; 4] = [0; 4];
        lock_time_bytes.copy_from_slice(&bytes[*offset..(*offset + 4)]);
        *offset += 4;
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

    pub fn marshalling(&self, bytes: &mut Vec<u8>) {
        let version_bytes: [u8; 4] = self.version.to_le_bytes();
        bytes.extend_from_slice(&version_bytes);
        bytes.extend_from_slice(&self.txin_count.marshalling());
        for tx_in in &self.tx_in {
            tx_in.marshalling(bytes);
        }
        bytes.extend_from_slice(&self.txout_count.marshalling());
        for tx_out in &self.tx_out {
            tx_out.marshalling(bytes);
        }
        let locktime_bytes: [u8; 4] = self.lock_time.to_le_bytes();
        bytes.extend_from_slice(&locktime_bytes);
    }

    pub fn hash(&self) -> [u8; 32] {
        let mut raw_transaction_bytes: Vec<u8> = Vec::new();
        self.marshalling(&mut raw_transaction_bytes);
        let hash_transaction = sha256d::Hash::hash(&raw_transaction_bytes);
        *hash_transaction.as_byte_array()
    }

    pub fn unmarshalling_transactions(
        bytes: &Vec<u8>,
        amount_transactions: u64,
        offset: &mut usize,
    ) -> Result<Vec<Transaction>, &'static str> {
        let mut transactions_list: Vec<Transaction> = Vec::new();
        let mut i = 0;
        while i < amount_transactions {
            transactions_list.push(Self::unmarshalling(bytes, offset)?);
            i += 1;
        }
        Ok(transactions_list)
    }
    pub fn is_coinbase_transaction(&self) -> bool {
        self.tx_in[0].is_coinbase()
    }
    pub fn get_txout(&self) -> Vec<TxOut> {
        self.tx_out.clone()
    }
    /// funcion que se encarga de remover las utxos usadas por esta tx
    pub fn remove_utxos(&self, container: &mut Vec<UtxoTuple>) {
        for list_utxos in container {
            for tx_in in &self.tx_in {
                // aca nos fijamos si alguna de nuestra inputs usa outputs anteriores
                // si la usa debemos remover dicho elemento de la lista
                if tx_in.is_same_hash(&list_utxos.hash) {
                    let mut position: usize = 0;
                    while position < list_utxos.utxo_set.len() {
                        if list_utxos.utxo_set[position].1 == tx_in.previous_index() {
                            list_utxos.utxo_set.remove(position);
                        }
                        position += 1;
                    }
                }
            }
        }
    }
    pub fn load_utxos(&self, container: &mut Vec<UtxoTuple>) {
        let hash = self.hash();
        let mut utxos_and_index = Vec::new();
        let position: usize = 0;
        for utxo in &self.tx_out {
            let utxo_and_index = (utxo.clone(), position);
            utxos_and_index.push(utxo_and_index);
        }
        let utxo_tuple = UtxoTuple::new(hash, utxos_and_index);
        container.push(utxo_tuple);
    }

    pub fn generate_transaction_to(
        account_sender: Account,
        address_receiver: &str,
        value: i64,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

#[cfg(test)]

mod test {
    use super::Transaction;
    use crate::{
        compact_size_uint::CompactSizeUint,
        transactions::{outpoint::Outpoint, sig_script::SigScript, tx_in::TxIn, tx_out::TxOut},
    };
    use bitcoin_hashes::{sha256d, Hash};

    fn crear_txins(cantidad: u128) -> Vec<TxIn> {
        let mut tx_in: Vec<TxIn> = Vec::new();
        for _i in 0..cantidad {
            let tx_id: [u8; 32] = [1; 32];
            let index_outpoint: u32 = 0x30000000;
            let outpoint: Outpoint = Outpoint::new(tx_id, index_outpoint);
            let compact_txin: CompactSizeUint = CompactSizeUint::new(1);
            let bytes: Vec<u8> = vec![1];
            let signature_script = SigScript::new(bytes);
            let sequence: u32 = 0xffffffff;
            tx_in.push(TxIn::new(
                outpoint,
                compact_txin,
                None,
                signature_script,
                sequence,
            ));
        }
        tx_in
    }

    fn crear_txouts(cantidad: u128) -> Vec<TxOut> {
        let mut tx_out: Vec<TxOut> = Vec::new();
        for _i in 0..cantidad {
            let value: i64 = 43;
            let pk_script_bytes: CompactSizeUint = CompactSizeUint::new(0);
            let pk_script: Vec<u8> = Vec::new();
            tx_out.push(TxOut::new(value, pk_script_bytes, pk_script));
        }
        tx_out
    }

    fn generar_flujo_de_datos(
        version: i32,
        tx_in_count: u128,
        tx_out_count: u128,
        lock_time: u32,
    ) -> Vec<u8> {
        //contenedor de bytes
        let mut bytes: Vec<u8> = Vec::new();
        // version settings
        let version: i32 = version;
        // tx_in_count settings
        let txin_count = CompactSizeUint::new(tx_in_count);
        // tx_in settings
        let tx_in: Vec<TxIn> = crear_txins(tx_in_count);
        // tx_out_count settings
        let txout_count = CompactSizeUint::new(tx_out_count);
        // tx_out settings
        let tx_out: Vec<TxOut> = crear_txouts(tx_out_count);
        //lock_time settings
        let lock_time: u32 = lock_time;
        let transaction: Transaction =
            Transaction::new(version, txin_count, tx_in, txout_count, tx_out, lock_time);
        transaction.marshalling(&mut bytes);
        bytes
    }

    #[test]
    fn test_la_transaccion_se_hashea_correctamente() {
        let previous_output: Outpoint = Outpoint::new([1; 32], 0x11111111);
        let script_bytes: CompactSizeUint = CompactSizeUint::new(0);
        let mut tx_in: Vec<TxIn> = Vec::new();
        tx_in.push(TxIn::new(
            previous_output,
            script_bytes,
            None,
            SigScript::new(Vec::new()),
            0x11111111,
        ));
        let pk_script_bytes: CompactSizeUint = CompactSizeUint::new(0);
        let mut tx_out: Vec<TxOut> = Vec::new();
        tx_out.push(TxOut::new(0x1111111111111111, pk_script_bytes, Vec::new()));
        let txin_count: CompactSizeUint = CompactSizeUint::new(1);
        let txout_count: CompactSizeUint = CompactSizeUint::new(1);
        let transaction: Transaction = Transaction::new(
            0x11111111,
            txin_count,
            tx_in,
            txout_count,
            tx_out,
            0x11111111,
        );
        let mut vector = Vec::new();
        transaction.marshalling(&mut vector);
        let hash_transaction = sha256d::Hash::hash(&vector);
        assert_eq!(transaction.hash(), *hash_transaction.as_byte_array());
    }

    #[test]
    fn test_unmarshalling_transaction_invalida() {
        let bytes: Vec<u8> = vec![0; 5];

        let mut offset: usize = 0;
        let transaction = Transaction::unmarshalling(&bytes, &mut offset);
        assert!(transaction.is_err());
    }

    #[test]
    fn test_unmarshalling_transaction_con_coinbase_y_mas_inputs_devuelve_error() {
        //contenedor de bytes
        let mut bytes: Vec<u8> = Vec::new();
        // version settings
        let version: i32 = 23;
        let version_bytes = version.to_le_bytes();
        bytes.extend_from_slice(&version_bytes[0..4]);
        // tx_in_count settings
        let txin_count = CompactSizeUint::new(2);
        bytes.extend_from_slice(&txin_count.marshalling()[0..1]);
        // tx_in settings
        let tx_id: [u8; 32] = [0; 32];
        let index_outpoint: u32 = 0xffffffff;
        let outpoint: Outpoint = Outpoint::new(tx_id, index_outpoint);
        let compact_txin: CompactSizeUint = CompactSizeUint::new(5);
        let height = Some(vec![1, 1, 1, 1]);
        let bytes_to_sig: Vec<u8> = vec![1];
        let signature_script = SigScript::new(bytes_to_sig);
        let sequence: u32 = 0xffffffff;
        let mut tx_in: Vec<TxIn> = Vec::new();
        tx_in.push(TxIn::new(
            outpoint,
            compact_txin,
            height,
            signature_script,
            sequence,
        ));
        tx_in[0 as usize].marshalling(&mut bytes);
        let cantidad_txin: u128 = txin_count.decoded_value() as u128;
        let tx_input: Vec<TxIn> = crear_txins(cantidad_txin);
        tx_input[0 as usize].marshalling(&mut bytes);
        // tx_out_count settings
        let txout_count = CompactSizeUint::new(1);
        bytes.extend_from_slice(txout_count.value());
        // tx_out settings
        let cantidad_txout: u128 = txout_count.decoded_value() as u128;
        let tx_out: Vec<TxOut> = crear_txouts(cantidad_txout);
        tx_out[0 as usize].marshalling(&mut bytes);
        //lock_time settings
        let lock_time: [u8; 4] = [0; 4];
        bytes.extend_from_slice(&lock_time);

        let mut offset: usize = 0;
        let transaction: Result<Transaction, &'static str> =
            Transaction::unmarshalling(&bytes, &mut offset);
        assert!(transaction.is_err());
    }

    #[test]
    fn test_unmarshalling_transaction_devuelve_version_esperado() -> Result<(), &'static str> {
        let tx_in_count: u128 = 4;
        let tx_out_count: u128 = 3;
        let version: i32 = -34;
        let lock_time: u32 = 3;
        let bytes = generar_flujo_de_datos(version, tx_in_count, tx_out_count, lock_time);
        let mut offset: usize = 0;
        let transaction: Transaction = Transaction::unmarshalling(&bytes, &mut offset)?;
        assert_eq!(transaction.version, version);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_transaction_devuelve_txin_count_esperado() -> Result<(), &'static str> {
        let tx_in_count: u128 = 4;
        let tx_out_count: u128 = 3;
        let version: i32 = -34;
        let lock_time: u32 = 3;
        let bytes = generar_flujo_de_datos(version, tx_in_count, tx_out_count, lock_time);
        let mut offset: usize = 0;
        let transaction: Transaction = Transaction::unmarshalling(&bytes, &mut offset)?;
        let tx_count_expected: CompactSizeUint = CompactSizeUint::new(tx_in_count);
        assert_eq!(transaction.txin_count, tx_count_expected);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_transaction_devuelve_txin_esperado() -> Result<(), &'static str> {
        let tx_in_count: u128 = 1;
        let tx_out_count: u128 = 1;
        let version: i32 = -34;
        let lock_time: u32 = 3;
        let bytes = generar_flujo_de_datos(version, tx_in_count, tx_out_count, lock_time);
        let mut offset: usize = 0;
        let transaction: Transaction = Transaction::unmarshalling(&bytes, &mut offset)?;
        let tx_in: Vec<TxIn> = crear_txins(tx_in_count);
        assert_eq!(transaction.tx_in, tx_in);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_transaction_devuelve_txout_count_esperado() -> Result<(), &'static str> {
        let tx_in_count: u128 = 4;
        let tx_out_count: u128 = 3;
        let version: i32 = -34;
        let lock_time: u32 = 3;
        let bytes = generar_flujo_de_datos(version, tx_in_count, tx_out_count, lock_time);
        let mut offset: usize = 0;
        let transaction: Transaction = Transaction::unmarshalling(&bytes, &mut offset)?;
        let tx_count_expected: CompactSizeUint = CompactSizeUint::new(tx_out_count);
        assert_eq!(transaction.txout_count, tx_count_expected);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_transaction_devuelve_txout_esperado() -> Result<(), &'static str> {
        let tx_in_count: u128 = 1;
        let tx_out_count: u128 = 1;
        let version: i32 = -34;
        let lock_time: u32 = 3;
        let bytes = generar_flujo_de_datos(version, tx_in_count, tx_out_count, lock_time);
        let mut offset: usize = 0;
        let transaction: Transaction = Transaction::unmarshalling(&bytes, &mut offset)?;
        let tx_out: Vec<TxOut> = crear_txouts(tx_out_count);
        assert_eq!(transaction.tx_out[0], tx_out[0]);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_transaction_devuelve_lock_time_esperado() -> Result<(), &'static str> {
        let tx_in_count: u128 = 4;
        let tx_out_count: u128 = 3;
        let version: i32 = -34;
        let lock_time: u32 = 3;
        let bytes = generar_flujo_de_datos(version, tx_in_count, tx_out_count, lock_time);
        let mut offset: usize = 0;
        let transaction: Transaction = Transaction::unmarshalling(&bytes, &mut offset)?;
        assert_eq!(transaction.lock_time, lock_time);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_transaction_devuelve_tamanio_txin_esperado() -> Result<(), &'static str> {
        let tx_in_count: u128 = 4;
        let tx_out_count: u128 = 3;
        let version: i32 = -34;
        let lock_time: u32 = 3;
        let bytes = generar_flujo_de_datos(version, tx_in_count, tx_out_count, lock_time);
        let mut offset: usize = 0;
        let transaction: Transaction = Transaction::unmarshalling(&bytes, &mut offset)?;
        assert_eq!(transaction.tx_in.len(), tx_in_count as usize);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_transaction_devuelve_vector_txin_esperado() -> Result<(), &'static str> {
        let tx_in_count: u128 = 4;
        let tx_out_count: u128 = 3;
        let version: i32 = -34;
        let lock_time: u32 = 3;
        let bytes = generar_flujo_de_datos(version, tx_in_count, tx_out_count, lock_time);
        let mut offset: usize = 0;
        let transaction: Transaction = Transaction::unmarshalling(&bytes, &mut offset)?;
        let tx_in: Vec<TxIn> = crear_txins(tx_in_count);
        assert_eq!(transaction.tx_in, tx_in);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_transaction_devuelve_vector_txout_esperado() -> Result<(), &'static str> {
        let tx_in_count: u128 = 4;
        let tx_out_count: u128 = 3;
        let version: i32 = -34;
        let lock_time: u32 = 3;
        let bytes = generar_flujo_de_datos(version, tx_in_count, tx_out_count, lock_time);
        let mut offset: usize = 0;
        let transaction: Transaction = Transaction::unmarshalling(&bytes, &mut offset)?;
        let tx_out: Vec<TxOut> = crear_txouts(tx_out_count);
        assert_eq!(transaction.tx_out, tx_out);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_de_2_transactions_devuelve_longitud_esperada() -> Result<(), &'static str>
    {
        let tx_in_count: u128 = 1;
        let tx_out_count: u128 = 1;
        let version: i32 = -34;
        let lock_time: u32 = 3;
        let mut bytes = generar_flujo_de_datos(version, tx_in_count, tx_out_count, lock_time);
        let bytes2 = generar_flujo_de_datos(version, tx_in_count, tx_out_count, lock_time);
        bytes.extend_from_slice(&bytes2[0..bytes2.len()]);
        let mut offset: usize = 0;
        let transaction: Vec<Transaction> =
            Transaction::unmarshalling_transactions(&bytes, 2, &mut offset)?;
        assert_eq!(transaction.len(), 2);
        Ok(())
    }
}
