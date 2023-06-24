use std::{
    collections::HashMap,
    error::Error,
    io,
    sync::{Arc, RwLock},
};

use bitcoin::{
    blocks::{block::Block, block_header::BlockHeader},
    compact_size_uint::CompactSizeUint,
    custom_errors::NodeCustomErrors,
    transactions::{
        outpoint::Outpoint, script::sig_script::SigScript, transaction::Transaction, tx_in::TxIn,
        tx_out::TxOut,
    },
    utxo_tuple::UtxoTuple,
};

type UtxoSetPointer = Arc<RwLock<HashMap<[u8; 32], UtxoTuple>>>;

fn create_txout(value: i64) -> TxOut {
    let pk_script_bytes: CompactSizeUint = CompactSizeUint::new(1);
    let pk_script: Vec<u8> = vec![1];
    TxOut::new(value, pk_script_bytes, pk_script)
}

fn create_tx_outs(values: Vec<i64>) -> Vec<TxOut> {
    let mut tx_outs: Vec<TxOut> = Vec::new();
    for value in values {
        tx_outs.push(create_txout(value));
    }
    tx_outs
}

fn create_txin(previous_output: Outpoint, height: Option<Vec<u8>>) -> TxIn {
    let script_bytes: CompactSizeUint = CompactSizeUint::new(1);
    let bytes: Vec<u8> = vec![1];
    let signature_script = SigScript::new(bytes);
    let sequence: u32 = 0x20202020;
    TxIn::new(
        previous_output,
        script_bytes,
        height,
        signature_script,
        sequence,
    )
}

fn create_txins(hashes: Vec<[u8; 32]>, indexs: Vec<u32>, tx_in: &mut Vec<TxIn>) {
    let mut outpoints: Vec<Outpoint> = Vec::new();
    let lenght: usize = hashes.len();
    for i in 0..lenght {
        outpoints.push(Outpoint::new(hashes[i], indexs[i]));
    }
    for outpoint in outpoints {
        let new_tx_in = create_txin(outpoint, None);
        tx_in.push(new_tx_in);
    }
}

fn create_coinbase_output() -> Outpoint {
    let tx_id: [u8; 32] = [0; 32];
    let index: u32 = 0xffffffff;
    Outpoint::new(tx_id, index)
}

fn create_transaction(
    txin_count: CompactSizeUint,
    tx_in: Vec<TxIn>,
    txout_count: CompactSizeUint,
    tx_out: Vec<TxOut>,
) -> Transaction {
    let version: i32 = 0x00000001;
    let lock_time: u32 = 0x02030405;
    Transaction::new(version, txin_count, tx_in, txout_count, tx_out, lock_time)
}

fn create_block_header() -> BlockHeader {
    BlockHeader {
        version: (0x30),
        previous_block_header_hash: ([0; 32]),
        merkle_root_hash: ([0; 32]),
        time: (0x01),
        n_bits: (0x10),
        nonce: (0x20),
    }
}

#[test]
fn test_lista_de_utxos_de_un_bloque_con_2_transacciones_tiene_largo_esperado(
) -> Result<(), Box<dyn Error>> {
    // coinbase transaction
    // seteo de tx_outs de la coinbase
    let coinbase_values_tx_outs: Vec<i64> = vec![1000, 200, 500];
    let txout_count: CompactSizeUint = CompactSizeUint::new(3);
    let tx_out: Vec<TxOut> = create_tx_outs(coinbase_values_tx_outs);
    // seteo de tx_ins de la coinbase
    let mut tx_in: Vec<TxIn> = Vec::new();
    let txin_count: CompactSizeUint = CompactSizeUint::new(1);
    let coinbase_output: Outpoint = create_coinbase_output();
    let coinbase_height: Option<Vec<u8>> = Some(vec![1, 2]);
    tx_in.push(create_txin(coinbase_output, coinbase_height));
    // creacion de la coinbase transaction
    let coinbase_transaction: Transaction =
        create_transaction(txin_count, tx_in, txout_count, tx_out);
    // primer transaction despues de la coinbase
    // seteo de tx_out de la transaccion
    let coinbase_values_tx_outs: Vec<i64> = vec![1000, 200, 500];
    let txout_count: CompactSizeUint = CompactSizeUint::new(3);
    let tx_out: Vec<TxOut> = create_tx_outs(coinbase_values_tx_outs);
    // seteo de tx_in de la transaccion
    let mut hashes: Vec<[u8; 32]> = Vec::new();
    let coinbase_hash: [u8; 32] = coinbase_transaction.hash();
    hashes.push(coinbase_hash);
    hashes.push(coinbase_hash);
    let indexs: Vec<u32> = vec![0, 1];
    let txin_count: CompactSizeUint = CompactSizeUint::new(2);
    let mut tx_in: Vec<TxIn> = Vec::new();
    create_txins(hashes, indexs, &mut tx_in);
    // creacion de la transaccion
    let first_transaction: Transaction = create_transaction(txin_count, tx_in, txout_count, tx_out);
    //creacion del bloque
    let mut txn: Vec<Transaction> = Vec::new();
    let txn_count: CompactSizeUint = CompactSizeUint::new(2);
    txn.push(coinbase_transaction);
    txn.push(first_transaction);

    let block: Block = Block {
        block_header: (create_block_header()),
        txn_count,
        txn,
    };
    let pointer_to_utxo_set: UtxoSetPointer = Arc::new(RwLock::new(HashMap::new()));

    block
        .give_me_utxos(pointer_to_utxo_set.clone())
        .map_err(|err| NodeCustomErrors::LockError(err.to_string()))?;
    let mut amount_utxos = 0;
    let utxo_set = match pointer_to_utxo_set.read() {
        Ok(utxo_set) => utxo_set,
        Err(_) => {
            return Err(Box::new(std::io::Error::new(
                io::ErrorKind::Other,
                "Falló al leer el puntero del utxo set",
            )));
        }
    };
    for utxo_tuple in utxo_set.values() {
        amount_utxos += utxo_tuple.utxo_set.len();
    }

    // se esperan 4 transacciones ya que se usan las 2 primeras de la coinbase(utxos)
    // y de la primera no se utiliza ninguna utxo
    assert_eq!(amount_utxos, 4);
    Ok(())
}
