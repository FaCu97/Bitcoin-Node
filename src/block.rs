use crate::{
    block_header::BlockHeader, compact_size_uint::CompactSizeUint, transaction::Transaction,
};

#[derive(Debug, Clone)]
pub struct Block {
    pub block_header: BlockHeader,
    pub txn_count: CompactSizeUint,
    pub txn: Vec<Transaction>,
}

impl Block {
    pub fn unmarshalling(bytes: &Vec<u8>, offset: &mut usize) -> Result<Block, &'static str> {
        let block_header: BlockHeader = BlockHeader::unmarshalling(bytes, offset);
        let txn_count: CompactSizeUint = CompactSizeUint::unmarshalling(bytes, offset);
        let amount_transaction: u64 = txn_count.decoded_value();
        let txn: Vec<Transaction> =
            Transaction::unmarshalling_transactions(bytes, amount_transaction, offset)?;
        Ok(Block {
            block_header,
            txn_count,
            txn,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{
        block_header::BlockHeader, compact_size_uint::CompactSizeUint, outpoint::Outpoint,
        transaction::Transaction, tx_in::TxIn, tx_out::TxOut,
    };

    use super::Block;

    fn crear_txins(cantidad: u128) -> Vec<TxIn> {
        let mut tx_in: Vec<TxIn> = Vec::new();
        for _i in 0..cantidad {
            let tx_id: [u8; 32] = [1; 32];
            let index_outpoint: u32 = 0x30000000;
            let outpoint: Outpoint = Outpoint::new(tx_id, index_outpoint);
            let compact_txin: CompactSizeUint = CompactSizeUint::new(1);
            let signature_script: Vec<u8> = vec![1];
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

    fn crear_transaccion(
        version: i32,
        tx_in_count: u128,
        tx_out_count: u128,
        lock_time: u32,
    ) -> Transaction {
        //contenedor de bytes
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
        transaction
    }

    #[test]
    fn test_unmarshaling_del_bloque_genera_block_header_esperado() -> Result<(), &'static str> {
        let mut bytes_to_read: Vec<u8> = Vec::new();
        let block_header: BlockHeader = BlockHeader {
            version: (0x30201000),
            previous_block_header_hash: ([1; 32]),
            merkle_root_hash: ([2; 32]),
            time: (0x90807060),
            n_bits: (0x04030201),
            nonce: (0x30),
        };
        block_header.marshalling(&mut bytes_to_read);
        let txn_count_bytes: CompactSizeUint = CompactSizeUint::new(1);
        let txn_count: Vec<u8> = txn_count_bytes.marshalling();
        bytes_to_read.extend_from_slice(&txn_count);
        let tx_in_count: u128 = 4;
        let tx_out_count: u128 = 3;
        let version: i32 = -34;
        let lock_time: u32 = 3;
        let tx: Transaction = crear_transaccion(version, tx_in_count, tx_out_count, lock_time);
        tx.marshalling(&mut bytes_to_read);
        let mut offset: usize = 0;
        let block: Block = Block::unmarshalling(&bytes_to_read, &mut offset)?;
        assert_eq!(block.block_header, block_header);
        Ok(())
    }

    #[test]
    fn test_unmarshaling_del_bloque_genera_txn_count_esperado() -> Result<(), &'static str> {
        let mut bytes_to_read: Vec<u8> = Vec::new();
        let block_header: BlockHeader = BlockHeader {
            version: (0x30201000),
            previous_block_header_hash: ([1; 32]),
            merkle_root_hash: ([2; 32]),
            time: (0x90807060),
            n_bits: (0x04030201),
            nonce: (0x30),
        };
        block_header.marshalling(&mut bytes_to_read);
        let txn_count_bytes: CompactSizeUint = CompactSizeUint::new(1);
        let txn_count: Vec<u8> = txn_count_bytes.marshalling();
        bytes_to_read.extend_from_slice(&txn_count);
        let tx_in_count: u128 = 4;
        let tx_out_count: u128 = 3;
        let version: i32 = -34;
        let lock_time: u32 = 3;
        let tx: Transaction = crear_transaccion(version, tx_in_count, tx_out_count, lock_time);
        tx.marshalling(&mut bytes_to_read);
        let mut offset: usize = 0;
        let block: Block = Block::unmarshalling(&bytes_to_read, &mut offset)?;
        assert_eq!(block.txn_count, txn_count_bytes);
        Ok(())
    }

    #[test]
    fn test_unmarshaling_del_bloque_genera_transaction_esperada() -> Result<(), &'static str> {
        let mut bytes_to_read: Vec<u8> = Vec::new();
        let block_header: BlockHeader = BlockHeader {
            version: (0x30201000),
            previous_block_header_hash: ([1; 32]),
            merkle_root_hash: ([2; 32]),
            time: (0x90807060),
            n_bits: (0x04030201),
            nonce: (0x30),
        };
        block_header.marshalling(&mut bytes_to_read);
        let txn_count_bytes: CompactSizeUint = CompactSizeUint::new(1);
        let txn_count: Vec<u8> = txn_count_bytes.marshalling();
        bytes_to_read.extend_from_slice(&txn_count);
        let tx_in_count: u128 = 1;
        let tx_out_count: u128 = 1;
        let version: i32 = 100;
        let lock_time: u32 = 3;
        let tx: Transaction = crear_transaccion(version, tx_in_count, tx_out_count, lock_time);
        tx.marshalling(&mut bytes_to_read);
        let mut offset: usize = 0;
        let block: Block = Block::unmarshalling(&bytes_to_read, &mut offset)?;
        assert_eq!(block.txn[0], tx);
        Ok(())
    }
}
