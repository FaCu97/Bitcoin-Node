use crate::transactions::tx_out::TxOut;
#[derive(Debug, Clone)]
pub struct UtxoTuple {
    pub hash: [u8; 32],
    pub utxo_set: Vec<(TxOut, usize)>,
}

impl UtxoTuple {
    pub fn new(hash: [u8; 32], utxo_set: Vec<(TxOut, usize)>) -> Self {
        UtxoTuple { hash, utxo_set }
    }

    /// Devuelve la utxoTuple con las TxOut que referencian a la dirección recibida
    /// En caso de que no encuentre ninguna, devuelve None
    pub fn referenced_utxos(&self, address: &str) -> Option<UtxoTuple> {
        let hash = self.hash;
        let mut utxo_set: Vec<(TxOut, usize)> = Vec::new();
        for utxo in &self.utxo_set {
            match utxo.0.get_adress() {
                Ok(value) => {
                    if *address == value {
                        utxo_set.push(utxo.clone());
                    }
                }
                Err(_) => {
                    continue;
                }
            }
        }
        if utxo_set.is_empty() {
            return None;
        }
        Some(UtxoTuple { hash, utxo_set })
    }

    /// Devuelve el monto en satoshis de las TxOut del Utxo
    pub fn balance(&self) -> i64 {
        let mut balance = 0;
        for utxo in &self.utxo_set {
            balance += utxo.0.value();
        }
        balance
    }
    pub fn utxos_to_spend(&self, value: i64, partial_amount: &mut i64) -> UtxoTuple {
        let utxos_to_spend = Vec::new();
        for tx_out in &self.utxo_set {
            if *partial_amount > value {
                break;
            }
        }
        Self::new(self.hash, utxos_to_spend)
    }
}
