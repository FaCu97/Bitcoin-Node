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

    pub fn hash(&self) -> [u8; 32] {
        self.hash.clone()
    }
    /// Funcion que se usa para la generacion de los txIn a la hora de crear una nueva transaccion
    /// puede suceder que una transaccion tenga mas de un outpoint referenciando a las utxos de esa
    /// transaccion por eso la necesidad de esta funcion
    pub fn get_indexes_from_utxos(&self) -> Vec<usize> {
        let mut indexes = Vec::new();
        for utxo in &self.utxo_set {
            indexes.push(utxo.1);
        }
        indexes
    }

    /// Recive el monto total a gastar, y monto que ya se juntó
    /// Remueve las utxos necesarias hasta llegar al monto total y las devuelve en un nuevo UtxoTuple
    pub fn utxos_to_spend(&mut self, value: i64, partial_amount: &mut i64) -> UtxoTuple {
        let mut utxos_to_spend = Vec::new();
        let mut position: usize = 0;
        let lenght: usize = self.utxo_set.len();
        while position < lenght {
            *partial_amount += self.utxo_set[position].0.value();
            // No corresponde removerlas mientras la tx no está confirmada
            // let utxo = self.utxo_set.remove(position);
            utxos_to_spend.push(self.utxo_set[position].clone());
            if *partial_amount > value {
                break;
            }
            position += 1;
        }
        Self::new(self.hash, utxos_to_spend)
    }

    pub fn find(&self, previous_hash: [u8; 32], previous_index: usize) -> Option<&Vec<u8>> {
        if self.hash != previous_hash {
            return None;
        }
        for utxo in &self.utxo_set {
            if utxo.1 == previous_index {
                return Some(utxo.0.get_pub_key());
            }
        }
        None
    }
}

#[cfg(test)]

mod test {}
