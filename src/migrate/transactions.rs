use anyhow::{Context, Result};
use std::collections::HashMap;
use zewif::{BlockHash, TxBlockPosition, TxId};

use crate::{ZcashdWallet, zcashd_wallet::WalletTx};

/// Convert ZCashd transactions to Zewif format
pub fn convert_transactions(wallet: &ZcashdWallet) -> Result<HashMap<TxId, zewif::Transaction>> {
    let mut transactions = HashMap::new();

    for (tx_id, wallet_tx) in wallet.transactions() {
        let zewif_tx = convert_transaction(*tx_id, wallet_tx)
            .with_context(|| format!("Failed to convert transaction {}", tx_id))?;
        transactions.insert(*tx_id, zewif_tx);
    }

    Ok(transactions)
}

/// Convert a single ZCashd transaction to Zewif format
fn convert_transaction(tx_id: TxId, tx: &WalletTx) -> Result<zewif::Transaction> {
    let mut zewif_tx = zewif::Transaction::new(tx_id);

    // Set raw transaction data
    if !tx.unparsed_data().is_empty() {
        zewif_tx.set_raw(tx.unparsed_data().clone());
    }

    // Add transaction metadata

    // Extract block hash if available
    if tx.hash_block() != BlockHash::from_bytes([0u8; 32]) {
        zewif_tx.set_block_position(Some(TxBlockPosition::new(
            tx.hash_block(),
            tx.index().try_into().unwrap(),
        )))
    };

    // TODO
    //    //
    //    // Access sapling note data hashmap for witness information if available
    //    let sapling_note_data = tx.sapling_note_data();
    //
    //    // Find the matching note data for this output if available
    //    if let Some(note_data_map) = sapling_note_data {
    //        for (outpoint, note_data) in note_data_map {
    //            // Match output by commitment and position in the transaction
    //            // (Finding exact output match may require more complex lookups in practice)
    //            if outpoint.vout() == idx as u32 && outpoint.txid() == tx_id {
    //                // Add witness data if available
    //                if !note_data.witnesses().is_empty() {
    //                    // Get the best witness (the last one)
    //                    if let Some(witness) = note_data.witnesses().last() {
    //                        // For the anchor, we would normally use the Merkle root
    //                        // Since we don't have direct access to it, we'll create a placeholder
    //                        // for now and improve it in a future implementation
    //                        let anchor = u256::default();
    //                        sapling_output.set_witness(Some(SaplingAnchorWitness::new(
    //                            anchor,
    //                            witness.clone(),
    //                        )));
    //                    }
    //                }
    //
    //                // We don't extract or decrypt memo fields during migration
    //                // The memo is inside the encrypted ciphertext, but we preserve
    //                // the whole ciphertext in the output description
    //                // The receiving wallet is responsible for decrypting
    //                // and extracting the memo with the appropriate keys
    //                sapling_output.set_memo(None);
    //
    //                break;
    //            }
    //        }
    //
    //        zewif_tx.add_sapling_output(sapling_output);
    //    }

    Ok(zewif_tx)
}
