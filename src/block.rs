use bitcoin_hashes::{sha256, Hash};

use crate::{
    block_header::BlockHeader, compact_size_uint::CompactSizeUint, transaction::Transaction,
};

#[derive(Debug)]
pub struct Block {
    block_header: BlockHeader,
    txn_count: CompactSizeUint,
    txn: Vec<Transaction>,
}

impl Block {
    pub fn new(
        block_header: BlockHeader,
        txn_count: CompactSizeUint,
        txn: Vec<Transaction>,
    ) -> Block {
        Block {
            block_header,
            txn_count,
            txn,
        }
    }

    pub fn unmarshalling(bytes: &Vec<u8>, offset: &mut usize) -> Result<Block, &'static str> {
        let block_header: BlockHeader = BlockHeader::unmarshalling(bytes, offset)?;
        let txn_count: CompactSizeUint = CompactSizeUint::unmarshalling(bytes, offset)?;
        let amount_transaction: u64 = txn_count.decoded_value();
        let txn: Vec<Transaction> =
            Transaction::unmarshalling_transactions(bytes, amount_transaction, offset)?;
        Ok(Block {
            block_header,
            txn_count,
            txn,
        })
    }
    // Esta funcion se encarga de validar el bloque , primero realiza la proof of work
    // luego realiza la proof of inclusion sobre su lista de transacciones
    pub fn validate(&mut self) -> (bool, &'static str) {
        //proof of work
        if !self.block_header.validate() {
            return (false, "El bloque no cumple con la dificultad pedida");
        }
        //proof of inclusion
        let merkle_root_hash: &[u8; 32] = &self.generate_merkle_root();
        if !self.block_header.is_same_merkle_root_hash(merkle_root_hash) {
            return (
                false,
                "El merkle root generado es distinto al provisto por el block header",
            );
        }
        (true, "El bloque es valido")
    }
    // Esta funcion se encarga de concatenar los hashes recibidos y luego hashearlos
    fn concatenate_and_hash(first_hash: [u8; 32], second_hash: [u8; 32]) -> [u8; 32] {
        let mut hashs_concatenated: [u8; 64] = [0; 64];
        hashs_concatenated[..32].copy_from_slice(&first_hash[..32]);
        hashs_concatenated[32..(32 + 32)].copy_from_slice(&second_hash[..32]);
        *sha256::Hash::hash(&hashs_concatenated).as_byte_array()
    }
    //funcion que se encarga de reducir los elementos del vector de tx_ids , agruparlos
    // de a pares hasearlos y guardarlos nuevamente en un vector el cual sera procesado
    // recursivamente hasta obtener el merkle root hash
    fn recursive_generation_merkle_root(vector: Vec<[u8; 32]>) -> [u8; 32] {
        let vec_length: usize = vector.len();
        if vec_length == 1 {
            return vector[0];
        }
        let mut upper_level: Vec<[u8; 32]> = Vec::new();
        let mut amount_hashs: usize = 0;
        let mut current_position: usize = 0;
        for tx in &vector {
            amount_hashs += 1;
            if amount_hashs == 2 {
                upper_level.push(Self::concatenate_and_hash(
                    vector[current_position - 1],
                    *tx,
                ));
                amount_hashs = 0;
            }
            current_position += 1;
        }
        // si el largo del vector es impar el ultimo elelmento debe concatenarse consigo
        // mismo y luego aplicarse la funcion de hash
        if (vec_length % 2) != 0 {
            upper_level.push(Self::concatenate_and_hash(
                vector[current_position - 1],
                vector[current_position - 1],
            ));
        }
        Self::recursive_generation_merkle_root(upper_level)
    }

    fn generate_merkle_root(&self) -> [u8; 32] {
        let mut merkle_transactions: Vec<[u8; 32]> = Vec::new();
        for tx in &self.txn {
            merkle_transactions.push(tx.hash());
        }
        Self::recursive_generation_merkle_root(merkle_transactions)
    }
    // Esta funcion realiza la SPV , asumimos que recibimos los restantes elementos para
    // construir el merkle root en el siguiente orden : desde las hojas hacia la raiz
    pub fn merkle_proof_of_inclusion(
        &self,
        transaction_to_find: &Transaction,
        vector_hash: Vec<[u8; 32]>,
    ) -> bool {
        let mut current_hash: [u8; 32] = transaction_to_find.hash();
        for hash in vector_hash {
            current_hash = Self::concatenate_and_hash(hash, current_hash);
        }
        current_hash == self.generate_merkle_root()
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
            tx_out.push(TxOut::new(value, pk_script_bytes, pk_script, true));
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

    #[test]
    fn test_merkle_root_de_un_bloque_con_2_transacciones_se_genera_correctamente() {
        let block_header: BlockHeader = BlockHeader {
            version: (0x30201000),
            previous_block_header_hash: ([1; 32]),
            merkle_root_hash: ([2; 32]),
            time: (0x90807060),
            n_bits: (0x04030201),
            nonce: (0x30),
        };
        let txn_count_bytes: CompactSizeUint = CompactSizeUint::new(2);
        let mut txn: Vec<Transaction> = Vec::new();
        let tx_in_count: u128 = 4;
        let tx_out_count: u128 = 3;
        let version: i32 = -34;
        let lock_time: u32 = 3;
        txn.push(crear_transaccion(
            version,
            tx_in_count,
            tx_out_count,
            lock_time,
        ));
        let tx_in_count: u128 = 5;
        let tx_out_count: u128 = 3;
        let version: i32 = 34;
        let lock_time: u32 = 3;
        txn.push(crear_transaccion(
            version,
            tx_in_count,
            tx_out_count,
            lock_time,
        ));
        let first_hash: [u8; 32] = txn[0].hash();
        let second_hash: [u8; 32] = txn[1].hash();
        let expected_hash = Block::concatenate_and_hash(first_hash, second_hash);
        let block: Block = Block::new(block_header, txn_count_bytes, txn);
        assert_eq!(block.generate_merkle_root(), expected_hash);
    }

    #[test]
    fn test_merkle_root_de_un_bloque_con_3_transacciones_se_genera_correctamente() {
        let block_header: BlockHeader = BlockHeader {
            version: (0x30201000),
            previous_block_header_hash: ([1; 32]),
            merkle_root_hash: ([2; 32]),
            time: (0x90807060),
            n_bits: (0x04030201),
            nonce: (0x30),
        };
        let txn_count_bytes: CompactSizeUint = CompactSizeUint::new(2);
        let mut txn: Vec<Transaction> = Vec::new();
        let tx_in_count: u128 = 4;
        let tx_out_count: u128 = 3;
        let version: i32 = -34;
        let lock_time: u32 = 3;
        txn.push(crear_transaccion(
            version,
            tx_in_count,
            tx_out_count,
            lock_time,
        ));
        let tx_in_count: u128 = 9;
        let tx_out_count: u128 = 3;
        let version: i32 = -34;
        let lock_time: u32 = 67;
        txn.push(crear_transaccion(
            version,
            tx_in_count,
            tx_out_count,
            lock_time,
        ));
        let tx_in_count: u128 = 4;
        let tx_out_count: u128 = 2;
        let version: i32 = 39;
        let lock_time: u32 = 3;
        txn.push(crear_transaccion(
            version,
            tx_in_count,
            tx_out_count,
            lock_time,
        ));
        let first_hash: [u8; 32] = txn[0].hash();
        let second_hash: [u8; 32] = txn[1].hash();
        let third_hash: [u8; 32] = txn[2].hash();
        let expected_hash_1 = Block::concatenate_and_hash(first_hash, second_hash);
        let expected_hash_2 = Block::concatenate_and_hash(third_hash, third_hash);
        let expected_hash_final = Block::concatenate_and_hash(expected_hash_1, expected_hash_2);
        let block: Block = Block::new(block_header, txn_count_bytes, txn);
        assert_eq!(block.generate_merkle_root(), expected_hash_final);
    }

    #[test]
    fn test_merkle_proof_of_inclusion_funciona_correctamente() {
        let block_header: BlockHeader = BlockHeader {
            version: (0x30201000),
            previous_block_header_hash: ([1; 32]),
            merkle_root_hash: ([2; 32]),
            time: (0x90807060),
            n_bits: (0x04030201),
            nonce: (0x30),
        };
        let txn_count_bytes: CompactSizeUint = CompactSizeUint::new(2);
        let mut txn: Vec<Transaction> = Vec::new();
        let tx_in_count: u128 = 4;
        let tx_out_count: u128 = 3;
        let version: i32 = -34;
        let lock_time: u32 = 3;
        txn.push(crear_transaccion(
            version,
            tx_in_count,
            tx_out_count,
            lock_time,
        ));
        let tx_in_count: u128 = 9;
        let tx_out_count: u128 = 3;
        let version: i32 = -34;
        let lock_time: u32 = 67;
        txn.push(crear_transaccion(
            version,
            tx_in_count,
            tx_out_count,
            lock_time,
        ));
        let tx_in_count: u128 = 4;
        let tx_out_count: u128 = 2;
        let version: i32 = 39;
        let lock_time: u32 = 3;
        txn.push(crear_transaccion(
            version,
            tx_in_count,
            tx_out_count,
            lock_time,
        ));
        let tx_in_count: u128 = 4;
        let tx_out_count: u128 = 5;
        let version: i32 = 933;
        let lock_time: u32 = 2;
        txn.push(crear_transaccion(
            version,
            tx_in_count,
            tx_out_count,
            lock_time,
        ));
        let first_hash: [u8; 32] = txn[0].hash();
        let second_hash: [u8; 32] = txn[1].hash();
        let third_hash: [u8; 32] = txn[2].hash();
        let expected_hash_1 = Block::concatenate_and_hash(first_hash, second_hash);
        let mut vector: Vec<[u8; 32]> = Vec::new();
        vector.push(third_hash);
        vector.push(expected_hash_1);

        let block: Block = Block::new(block_header, txn_count_bytes, txn);
        let hola = &block.txn[3];
        assert!(block.merkle_proof_of_inclusion(hola, vector));
    }
}
