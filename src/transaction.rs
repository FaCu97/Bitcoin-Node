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
        if tx_in[0].is_coinbase() && txin_count.decoded_value() != 1{
            return Err(
                "una coinbase transaction no puede tener mas de un input",
            );         
        }
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

    pub fn marshalling(&self,bytes:&mut Vec<u8>){
        let version_bytes : [u8;4] = self.version.to_le_bytes();
        bytes.extend_from_slice(&version_bytes);
        bytes.extend_from_slice(&self.txin_count.marshalling());
        for tx_in in &self.tx_in{
            tx_in.marshalling(bytes);
        }
        bytes.extend_from_slice(&self.txout_count.marshalling());
        for tx_out in &self.tx_out{
            tx_out.marshalling(bytes);
        }
        let locktime_bytes : [u8;4] = self.lock_time.to_le_bytes();
        bytes.extend_from_slice(&locktime_bytes);


    }
}
#[cfg(test)]

mod test {
    use super::Transaction;
    use crate::{
        compact_size_uint::CompactSizeUint, outpoint::Outpoint, tx_in::TxIn, tx_out::TxOut,
    };

    fn crear_txin_y_pasar_a_bytes(cantidad : u64,bytes: &mut Vec<u8>) -> Vec<TxIn>{
        let mut tx_in:Vec<TxIn> = Vec::new();
        for i in 0..cantidad{
            let tx_id : [u8;32] = [1;32];
            let index_outpoint : u32 = 0x30000000;
            let outpoint : Outpoint = Outpoint::new(tx_id,index_outpoint);
            let compact_txin : CompactSizeUint = CompactSizeUint::new(1);
            let signature_script : Vec<u8> = vec![1];
            let sequence : u32 = 0xffffffff;
            tx_in.push(TxIn::new(outpoint,compact_txin,None,signature_script,sequence));
            tx_in[i as usize].marshalling(bytes);
        }
        tx_in
        
    }

    fn crear_txout_y_pasar_a_bytes(cantidad: u64,bytes: &mut Vec<u8>) -> Vec<TxOut>{
        let mut tx_out:Vec<TxOut> = Vec::new();
        for i in 0..cantidad{
            let value : i64 = 43;
            let pk_script_bytes: CompactSizeUint = CompactSizeUint::new(0);
            let pk_script: Vec<u8> = Vec::new();
            tx_out.push(TxOut::new(value,pk_script_bytes,pk_script));
            tx_out[i as usize].marshalling(bytes);
        }
        tx_out
    }

    #[test]
    fn test_unmarshalling_transaction_invalida(){
        let bytes : Vec<u8> = vec![0;5];
        let transaction = Transaction::unmarshalling(&bytes);
        assert!(transaction.is_err());
    }

    #[test]
    fn test_unmarshalling_transaction_con_coinbase_y_mas_inputs_devuelve_error(){
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
        let tx_id : [u8;32] = [0;32];
        let index_outpoint : u32 = 0xffffffff;
        let outpoint : Outpoint = Outpoint::new(tx_id,index_outpoint);
        let compact_txin : CompactSizeUint = CompactSizeUint::new(1);
        let signature_script : Vec<u8> = vec![1];
        let sequence : u32 = 0xffffffff;
        let mut tx_in : Vec<TxIn> = Vec::new();
        tx_in.push(TxIn::new(outpoint,compact_txin,None,signature_script,sequence));
        tx_in[0 as usize].marshalling(&mut bytes);
        let _tx_input : Vec<TxIn> = crear_txin_y_pasar_a_bytes(txin_count.decoded_value(),&mut bytes);
        // tx_out_count settings
        let txout_count = CompactSizeUint::new(1);
        bytes.extend_from_slice(txout_count.value());
        // tx_out settings
        let _tx_out:Vec<TxOut> = crear_txout_y_pasar_a_bytes(txout_count.decoded_value(),&mut bytes);
        //lock_time settings
        let lock_time: [u8; 4] = [0; 4];
        bytes.extend_from_slice(&lock_time);
        let transaction : Result<Transaction, &'static str> = Transaction::unmarshalling(&bytes);
        assert!(transaction.is_err());
    }

    #[test]
    fn test_unmarshalling_transaction_devuelve_version_esperado() -> Result<(), &'static str> {
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
        let _tx_in : Vec<TxIn> = crear_txin_y_pasar_a_bytes(txin_count.decoded_value(),&mut bytes);
        // tx_out_count settings
        let txout_count = CompactSizeUint::new(1);
        bytes.extend_from_slice(txout_count.value());
        // tx_out settings
        let _tx_out:Vec<TxOut> = crear_txout_y_pasar_a_bytes(txout_count.decoded_value(),&mut bytes);
        //lock_time settings
        let lock_time: [u8; 4] = [0; 4];
        bytes.extend_from_slice(&lock_time);
        let transaction : Transaction = Transaction::unmarshalling(&bytes)?;
        assert_eq!(version,transaction.version);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_transaction_devuelve_txin_count_esperado() -> Result<(), &'static str> {
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
        let _tx_in : Vec<TxIn> = crear_txin_y_pasar_a_bytes(txin_count.decoded_value(),&mut bytes);
        // tx_out_count settings
        let txout_count = CompactSizeUint::new(1);
        bytes.extend_from_slice(txout_count.value());
        // tx_out settings
        let _tx_out:Vec<TxOut> = crear_txout_y_pasar_a_bytes(txout_count.decoded_value(),&mut bytes);
        //lock_time settings
        let lock_time: [u8; 4] = [0; 4];
        bytes.extend_from_slice(&lock_time);
        let transaction : Transaction = Transaction::unmarshalling(&bytes)?;
        assert_eq!(txin_count,transaction.txin_count);
        Ok(())
    }
    
    #[test]
    fn test_unmarshalling_transaction_devuelve_txin_esperado() -> Result<(), &'static str> {
        //contenedor de bytes
        let mut bytes : Vec<u8> = Vec::new();
        // version settings
        let version : i32 = 23;
        let version_bytes = version.to_le_bytes();
        bytes.extend_from_slice(&version_bytes[0..4]);
        // tx_in_count settings
        let txin_count = CompactSizeUint::new(1);
        bytes.extend_from_slice(&txin_count.marshalling()[0..1]);
        // tx_in settings
        let tx_in : Vec<TxIn> = crear_txin_y_pasar_a_bytes(txin_count.decoded_value(),&mut bytes);
        // tx_out_count settings
        let txout_count = CompactSizeUint::new(1);
        bytes.extend_from_slice(txout_count.value());
        // tx_out settings
        let _tx_out:Vec<TxOut> = crear_txout_y_pasar_a_bytes(txout_count.decoded_value(),&mut bytes);
        //lock_time settings
        let lock_time : [u8;4] = [0;4];
        bytes.extend_from_slice(&lock_time);
        let transaction : Transaction = Transaction::unmarshalling(&bytes)?;
        assert_eq!(tx_in,transaction.tx_in);
        Ok(())
    }
    
    #[test]
    fn test_unmarshalling_transaction_devuelve_txout_count_esperado() -> Result<(), &'static str> {
        //contenedor de bytes
        let mut bytes : Vec<u8> = Vec::new();
        // version settings
        let version : i32 = 23;
        let version_bytes = version.to_le_bytes();
        bytes.extend_from_slice(&version_bytes[0..4]);
        // tx_in_count settings
        let txin_count = CompactSizeUint::new(1);
        bytes.extend_from_slice(&txin_count.marshalling()[0..1]);
        // tx_in settings
        let _tx_in : Vec<TxIn> = crear_txin_y_pasar_a_bytes(txin_count.decoded_value(),&mut bytes);
        // tx_out_count settings
        let txout_count = CompactSizeUint::new(1);
        bytes.extend_from_slice(txout_count.value());
        // tx_out settings
        let _tx_out:Vec<TxOut> = crear_txout_y_pasar_a_bytes(txout_count.decoded_value(),&mut bytes);
        //lock_time settings
        let lock_time : [u8;4] = [0;4];
        bytes.extend_from_slice(&lock_time);
        let transaction : Transaction = Transaction::unmarshalling(&bytes)?;
        assert_eq!(txout_count,transaction.txout_count);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_transaction_devuelve_txout_esperado() -> Result<(), &'static str> {
        //contenedor de bytes
        let mut bytes : Vec<u8> = Vec::new();
        // version settings
        let version : i32 = 23;
        let version_bytes = version.to_le_bytes();
        bytes.extend_from_slice(&version_bytes[0..4]);
        // tx_in_count settings
        let txin_count = CompactSizeUint::new(1);
        bytes.extend_from_slice(&txin_count.marshalling()[0..1]);
        // tx_in settings
        let _tx_in : Vec<TxIn> = crear_txin_y_pasar_a_bytes(txin_count.decoded_value(),&mut bytes);
        // tx_out_count settings
        let txout_count = CompactSizeUint::new(1);
        bytes.extend_from_slice(txout_count.value());
        // tx_out settings
        let tx_out:Vec<TxOut> = crear_txout_y_pasar_a_bytes(txout_count.decoded_value(),&mut bytes);
        //lock_time settings
        let lock_time : [u8;4] = [0;4];
        bytes.extend_from_slice(&lock_time);
        let transaction : Transaction = Transaction::unmarshalling(&bytes)?;
        assert_eq!(tx_out,transaction.tx_out);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_transaction_devuelve_lock_time_esperado() -> Result<(), &'static str> {
        //contenedor de bytes
        let mut bytes : Vec<u8> = Vec::new();
        // version settings
        let version : i32 = 23;
        let version_bytes = version.to_le_bytes();
        bytes.extend_from_slice(&version_bytes[0..4]);
        // tx_in_count settings
        let txin_count = CompactSizeUint::new(1);
        bytes.extend_from_slice(&txin_count.marshalling()[0..1]);
        // tx_in settings
        let _tx_in : Vec<TxIn> = crear_txin_y_pasar_a_bytes(txin_count.decoded_value(),&mut bytes);
        // tx_out_count settings
        let txout_count = CompactSizeUint::new(1);
        bytes.extend_from_slice(txout_count.value());
        // tx_out settings
        let _tx_out:Vec<TxOut> = crear_txout_y_pasar_a_bytes(txout_count.decoded_value(),&mut bytes);
        //lock_time settings
        let lock_time : [u8;4] = [0;4];
        bytes.extend_from_slice(&lock_time);
        let transaction : Transaction = Transaction::unmarshalling(&bytes)?;
        assert_eq!(0x00000000,transaction.lock_time);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_transaction_devuelve_tamanio_txin_esperado() -> Result<(), &'static str> {
        //contenedor de bytes
        let mut bytes : Vec<u8> = Vec::new();
        // version settings
        let version : i32 = 23;
        let version_bytes = version.to_le_bytes();
        bytes.extend_from_slice(&version_bytes[0..4]);
        // tx_in_count settings
        let txin_count = CompactSizeUint::new(2);
        bytes.extend_from_slice(&txin_count.marshalling()[0..1]);
        // tx_in settings
        let _tx_in : Vec<TxIn> = crear_txin_y_pasar_a_bytes(txin_count.decoded_value(),&mut bytes);
        // tx_out_count settings
        let txout_count = CompactSizeUint::new(1);
        bytes.extend_from_slice(txout_count.value());
        // tx_out settings
        let _tx_out:Vec<TxOut> = crear_txout_y_pasar_a_bytes(txout_count.decoded_value(),&mut bytes);
        //lock_time settings
        let lock_time : [u8;4] = [0;4];
        bytes.extend_from_slice(&lock_time);
        let transaction : Transaction = Transaction::unmarshalling(&bytes)?;
        assert_eq!(transaction.tx_in.len(),2);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_transaction_devuelve_vector_txin_esperado() -> Result<(), &'static str> {
        //contenedor de bytes
        let mut bytes : Vec<u8> = Vec::new();
        // version settings
        let version : i32 = 23;
        let version_bytes = version.to_le_bytes();
        bytes.extend_from_slice(&version_bytes[0..4]);
        // tx_in_count settings
        let txin_count = CompactSizeUint::new(2);
        bytes.extend_from_slice(&txin_count.marshalling()[0..1]);
        // tx_in settings
        let tx_in : Vec<TxIn> = crear_txin_y_pasar_a_bytes(txin_count.decoded_value(),&mut bytes);
        // tx_out_count settings
        let txout_count = CompactSizeUint::new(1);
        bytes.extend_from_slice(txout_count.value());
        // tx_out settings
        let _tx_out:Vec<TxOut> = crear_txout_y_pasar_a_bytes(txout_count.decoded_value(),&mut bytes);
        //lock_time settings
        let lock_time : [u8;4] = [0;4];
        bytes.extend_from_slice(&lock_time);
        let transaction : Transaction = Transaction::unmarshalling(&bytes)?;
        assert_eq!(transaction.tx_in,tx_in);
        Ok(())
    }

    #[test]
    fn test_unmarshalling_transaction_devuelve_vector_txout_esperado() -> Result<(), &'static str> {
        //contenedor de bytes
        let mut bytes : Vec<u8> = Vec::new();
        // version settings
        let version : i32 = 23;
        let version_bytes = version.to_le_bytes();
        bytes.extend_from_slice(&version_bytes[0..4]);
        // tx_in_count settings
        let txin_count = CompactSizeUint::new(2);
        bytes.extend_from_slice(&txin_count.marshalling()[0..1]);
        // tx_in settings
        let _tx_in : Vec<TxIn> = crear_txin_y_pasar_a_bytes(txin_count.decoded_value(),&mut bytes);
        // tx_out_count settings
        let txout_count = CompactSizeUint::new(3);
        bytes.extend_from_slice(txout_count.value());
        // tx_out settings
        let tx_out:Vec<TxOut> = crear_txout_y_pasar_a_bytes(txout_count.decoded_value(),&mut bytes);
        //lock_time settings
        let lock_time : [u8;4] = [0;4];
        bytes.extend_from_slice(&lock_time);
        let transaction : Transaction = Transaction::unmarshalling(&bytes)?;
        assert_eq!(transaction.tx_out,tx_out);
        Ok(())
    }
}
