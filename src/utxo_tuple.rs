use crate::transactions::tx_out::TxOut;

pub struct UtxoTuple {
    hash: [u8; 32],
    utxo_set: Vec<(TxOut, usize)>,
}

impl UtxoTuple {
    pub fn new(hash: [u8; 32], utxo_set: Vec<(TxOut, usize)>) -> Self {
        UtxoTuple { hash, utxo_set }
    }
}
