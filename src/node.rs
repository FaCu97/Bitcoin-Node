use crate::{block::Block, tx_out::TxOut};

pub struct Node{
    block_chain : Vec<Block>,
    utxo_set : Vec<TxOut>

}

impl Node{
    pub fn block_validation(block:Block) -> (bool,&'static str){
        block.validate();
        (false,"hola")
    }
    pub fn spend_an_utxo(value:i64) -> (bool,&'static str) {
       (false,"") 

    }
}
