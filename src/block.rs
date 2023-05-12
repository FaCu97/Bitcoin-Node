use bitcoin_hashes::{sha256, Hash};

use crate::{block_header::BlockHeader,compact_size_uint::CompactSizeUint,transaction::Transaction};



pub struct Block{
    block_header : BlockHeader,
    txn_count : CompactSizeUint,
    txn : Vec<Transaction>
}


impl Block{
    pub fn unmarshalling(bytes: &Vec<u8>,offset: &mut usize) -> Result<Block, &'static str>{
        let block_header : BlockHeader = BlockHeader::unmarshalling(bytes,offset);
        let txn_count : CompactSizeUint = CompactSizeUint::unmarshalling(bytes, offset);
        let amount_transaction : u64 = txn_count.decoded_value();
        let txn: Vec<Transaction> = Transaction::unmarshalling_transactions(bytes,amount_transaction,offset)?;
        Ok(Block { block_header, txn_count, txn })       
    }

    pub fn validate(&mut self) ->(bool,& 'static str){
        if !self.block_header.validate() {
            return(false,"El bloque no cumple con la dificultad pedida");
        }
        let merkle_root_hash : &[u8;32] = &self.generate_merkle_root();
        if  !self.block_header.is_same_merkle_root_hash(merkle_root_hash) {
            return (false,"El merkle root generado es distinto al provisto por el block header");
        }
        (true,"El bloque es valido")
        
    }
    fn concatenar_hash(primer_hash:[u8;32],segundo_hash:[u8;32]) -> [u8;64]{
        let mut hashs_concatenados:[u8;64] = [0;64];
        for i in 0..32{
            hashs_concatenados[i] = primer_hash[i]
        }
        for i in 0..32{
            hashs_concatenados[i + 32] = segundo_hash[i]
        }
        hashs_concatenados
    }

    fn generate_merkle_root_recursivo(vector: Vec<[u8;32]>) -> [u8;32]{
        if vector.len() == 1{
            return vector[0];
        }
        let vec_lenght : usize = vector.len();
        let contenedor_de_hashes : Vec<[u8;32]> = Vec::new();
        let mut amount_hashs:usize=1;
        let current_position:usize=0;
        for tx in &vector{
            if amount_hashs == 2{
                let hash_concatenados = Self::concatenar_hash(vector[current_position],*tx);
                let hash_transaction = sha256::Hash::hash(&hash_concatenados);

            }
        
            
        

        }

        let re: [u8;32] = [0;32];
        re
    }

    fn generate_merkle_root(&mut self) -> [u8;32]{
        let mut merkle_transactions :Vec<[u8;32]> = Vec::new();
        for tx in &mut self.txn{
            merkle_transactions.push(tx.hash());
        }
        Self::generate_merkle_root_recursivo(merkle_transactions)
    }


    
 /*   fn generate_merkle_root(&self)-> [u8;32]{
        let mut merkle_transactions :Vec<[u8;32]> = Vec::new();
        let vector_para_hashear : [u8;64] = Vec::new();
        for index in 0..self.txn.len(){
            merkle_transactions.push(self.txn[index].hash());
            if index%2 != 0{
                vector_para_hashear.extend_from_slice() 
            }
        }
        let mut i = 0;
        [0;32]

    }*/
}

#[cfg(test)]
mod test{
    use crate::{block_header::BlockHeader,outpoint::Outpoint, compact_size_uint::CompactSizeUint, tx_in::TxIn,tx_out::TxOut,transaction::Transaction};

    use super::Block;

    fn crear_txins(cantidad : u128,) -> Vec<TxIn>{
        let mut tx_in:Vec<TxIn> = Vec::new();
        for _i in 0..cantidad{
            let tx_id : [u8;32] = [1;32];
            let index_outpoint : u32 = 0x30000000;
            let outpoint : Outpoint = Outpoint::new(tx_id,index_outpoint);
            let compact_txin : CompactSizeUint = CompactSizeUint::new(1);
            let signature_script : Vec<u8> = vec![1];
            let sequence : u32 = 0xffffffff;
            tx_in.push(TxIn::new(outpoint,compact_txin,None,signature_script,sequence));
        }
        tx_in   
    }

    fn crear_txouts(cantidad: u128) -> Vec<TxOut>{
        let mut tx_out:Vec<TxOut> = Vec::new();
        for _i in 0..cantidad{
            let value : i64 = 43;
            let pk_script_bytes: CompactSizeUint = CompactSizeUint::new(0);
            let pk_script: Vec<u8> = Vec::new();
            tx_out.push(TxOut::new(value,pk_script_bytes,pk_script,true));
        }
        tx_out
    }

    fn crear_transaccion(version:i32,tx_in_count:u128,tx_out_count:u128,lock_time:u32)-> Transaction {
            //contenedor de bytes
            // version settings
            let version : i32 = version;
            // tx_in_count settings
            let txin_count = CompactSizeUint::new(tx_in_count);
            // tx_in settings
            let tx_in : Vec<TxIn> = crear_txins(tx_in_count);
            // tx_out_count settings
            let txout_count = CompactSizeUint::new(tx_out_count);
            // tx_out settings
            let tx_out:Vec<TxOut> = crear_txouts(tx_out_count);
            //lock_time settings
            let lock_time : u32 = lock_time;
            let transaction : Transaction = Transaction::new(version, txin_count, tx_in, txout_count, tx_out, lock_time);
            transaction
        }

    #[test]
    fn test_unmarshaling_del_bloque_genera_block_header_esperado()->Result<(), &'static str>{
        let mut bytes_to_read: Vec<u8> = Vec::new();
        let block_header:BlockHeader=BlockHeader { version: (0x30201000), previous_block_header_hash: ([1;32]), merkle_root_hash: ([2;32]), time: (0x90807060), n_bits: (0x04030201), nonce: (0x30) };
        block_header.marshalling(&mut bytes_to_read);
        let txn_count_bytes : CompactSizeUint = CompactSizeUint::new(1);
        let  txn_count : Vec<u8> = txn_count_bytes.marshalling();
        bytes_to_read.extend_from_slice(&txn_count);
        let tx_in_count :u128 = 4;
        let tx_out_count :u128 = 3;
        let version : i32 = -34;
        let lock_time : u32 = 3;
        let tx: Transaction = crear_transaccion(version,tx_in_count,tx_out_count,lock_time);
        tx.marshalling(&mut bytes_to_read);
        let mut offset:usize =0;
        let block : Block = Block::unmarshalling(&bytes_to_read,&mut offset)?;
        assert_eq!(block.block_header,block_header);
        Ok(())
    }

    #[test]
    fn test_unmarshaling_del_bloque_genera_txn_count_esperado()->Result<(), &'static str>{
        let mut bytes_to_read: Vec<u8> = Vec::new();
        let block_header:BlockHeader=BlockHeader { version: (0x30201000), previous_block_header_hash: ([1;32]), merkle_root_hash: ([2;32]), time: (0x90807060), n_bits: (0x04030201), nonce: (0x30) };
        block_header.marshalling(&mut bytes_to_read);
        let txn_count_bytes : CompactSizeUint = CompactSizeUint::new(1);
        let  txn_count : Vec<u8> = txn_count_bytes.marshalling();
        bytes_to_read.extend_from_slice(&txn_count);
        let tx_in_count :u128 = 4;
        let tx_out_count :u128 = 3;
        let version : i32 = -34;
        let lock_time : u32 = 3;
        let tx: Transaction = crear_transaccion(version,tx_in_count,tx_out_count,lock_time);
        tx.marshalling(&mut bytes_to_read);
        let mut offset:usize =0;
        let block : Block = Block::unmarshalling(&bytes_to_read,&mut offset)?;
        assert_eq!(block.txn_count,txn_count_bytes);
        Ok(())
    }

    #[test]
    fn test_unmarshaling_del_bloque_genera_transaction_esperada()->Result<(), &'static str>{
        let mut bytes_to_read: Vec<u8> = Vec::new();
        let block_header:BlockHeader=BlockHeader { version: (0x30201000), previous_block_header_hash: ([1;32]), merkle_root_hash: ([2;32]), time: (0x90807060), n_bits: (0x04030201), nonce: (0x30) };
        block_header.marshalling(&mut bytes_to_read);
        let txn_count_bytes : CompactSizeUint = CompactSizeUint::new(1);
        let  txn_count : Vec<u8> = txn_count_bytes.marshalling();
        bytes_to_read.extend_from_slice(&txn_count);
        let tx_in_count :u128 = 1;
        let tx_out_count :u128 = 1;
        let version : i32 = 100;
        let lock_time : u32 = 3;
        let tx: Transaction = crear_transaccion(version,tx_in_count,tx_out_count,lock_time);
        tx.marshalling(&mut bytes_to_read);
        let mut offset:usize =0;
        let block : Block = Block::unmarshalling(&bytes_to_read,&mut offset)?;
        assert_eq!(block.txn[0],tx);
        Ok(())
    }
}