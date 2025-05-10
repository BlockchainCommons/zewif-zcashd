use anyhow::Result;
use hex::ToHex;
use ripemd::{Digest, Ripemd160};
use sha2::Sha256;
use std::collections::HashSet;
use zewif::TxId;

use crate::{
    zcashd_wallet::{transparent::{KeyId, ScriptId}, u160, RecipientAddress, WalletTx}, ZcashdWallet
};

/// Extract all addresses involved in a transaction
pub fn extract_transaction_addresses(
    wallet: &ZcashdWallet,
    tx_id: TxId,
    tx: &WalletTx,
) -> Result<HashSet<String>> {
    let mut addresses = HashSet::new();
    let mut is_change_transaction = false;

    // Check if we have recipient mappings for this transaction
    if let Some(recipients) = wallet.send_recipients().get(&tx_id) {
        for recipient in recipients {
            // Add the unified address if it exists
            if !recipient.unified_address.is_empty() {
                addresses.insert(recipient.unified_address.clone());

                // Add a special tag to track unified addresses specifically
                addresses.insert(format!("ua:{}", recipient.unified_address.clone()));
            }

            // Add the recipient address based on the type
            match &recipient.recipient_address {
                RecipientAddress::Sapling(addr) => {
                    let addr_str = addr.to_string(wallet.network());
                    addresses.insert(addr_str.clone());
                    addresses.insert(format!("sapling_addr:{}", addr_str));
                }
                RecipientAddress::Orchard(addr) => {
                    let addr_str = addr.to_string(wallet.network());
                    addresses.insert(addr_str.clone());
                    addresses.insert(format!("orchard_addr:{}", addr_str));
                }
                RecipientAddress::KeyId(key_id) => {
                    // Convert P2PKH key hash to a Zcash address
                    let addr_str = key_id.to_string(wallet.network());
                    addresses.insert(addr_str.clone());
                    addresses.insert(format!("transparent_addr:{}", addr_str));
                }
                RecipientAddress::ScriptId(script_id) => {
                    // Convert P2SH script hash to a Zcash address
                    let addr_str = script_id.to_string(wallet.network());
                    addresses.insert(addr_str.clone());
                    addresses.insert(format!("transparent_script_addr:{}", addr_str));
                }
            }

            // Check if this is an internal address (change transaction)
            // FIXME: the following is not a valid way to detect change.
            //if !recipient.unified_address.is_empty() {
            //    if let Some(unified_accounts) = wallet.unified_accounts() {
            //        // Check if this unified address belongs to our wallet
            //        for addr_metadata in unified_accounts.address_metadata {
            //            // If we find this address in our metadata, it's likely a change address
            //            if format!("{}", addr_metadata.key_id) == recipient.unified_address {
            //                is_change_transaction = true;
            //                addresses.insert(format!("change:{}", recipient.unified_address));
            //                break;
            //            }
            //        }
            //    }
            //}
        }
    }

    // For transparent inputs, extract addresses from the script signatures
    if let Some(t_bundle) = tx.transaction().transparent_bundle() {
        for tx_in in t_bundle.vin.iter() {
            // Track the previous transaction
            let txid_str = format!("{}", tx_in.prevout.txid());
            let input_addr = format!("input:{}:{}", txid_str, tx_in.prevout.n());
            addresses.insert(input_addr);

            // Extract potential P2PKH or P2SH addresses from script signatures
            let script_data = tx_in.script_sig.0.clone();

            // For P2PKH signatures, extract the pubkey
            if script_data.len() > 33 {
                // Check for compressed pubkey
                let potential_pubkey = &script_data[script_data.len() - 33..];
                if potential_pubkey[0] == 0x02 || potential_pubkey[0] == 0x03 {
                    // Hash the pubkey to get the pubkey hash
                    let mut sha256 = Sha256::new();
                    sha256.update(potential_pubkey);
                    let sha256_result = sha256.finalize();

                    let mut ripemd160 = Ripemd160::new();
                    ripemd160.update(sha256_result);
                    let pubkey_hash = ripemd160.finalize();

                    // Create a transparent P2PKH address
                    let key_id = KeyId::from(
                        u160::from_slice(&pubkey_hash[..])
                            .expect("Creating u160 from RIPEMD160 hash"),
                    );
                    let addr_str = key_id.to_string(wallet.network());
                    addresses.insert(addr_str.clone());
                    addresses.insert(format!("transparent_spend:{}", addr_str));

                    // Check if this is one of our keys to better determine ownership
                    for key in wallet.keys().keypairs() {
                        // Cannot directly convert PubKey to Address, so we'll check differently
                        // Get the address from our address book that might match this key
                        for (address, _) in wallet.address_names().iter() {
                            if address.to_string() == addr_str {
                                addresses.insert(format!("our_key:{}", addr_str));

                                // If we have an HD path, we can determine if this is change
                                if let Some(hd_path) = key.metadata().hd_keypath() {
                                    if hd_path.contains("/1'/") || hd_path.contains("/1/") {
                                        // This is an internal key path, so this is likely change
                                        is_change_transaction = true;
                                        addresses.insert(format!("change_key:{}", addr_str));
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }

        // For transparent outputs, extract addresses
        for (vout_idx, tx_out) in t_bundle.vout.iter().enumerate() {
            let script_data = tx_out.script_pubkey.0.clone();
            let mut output_address = String::new();

            // P2PKH detection
            if script_data.len() >= 25 && script_data[0] == 0x76 && script_data[1] == 0xA9 {
                if script_data[23] == 0x88 && script_data[24] == 0xAC {
                    // Extract the pubkey hash and create an address
                    let pubkey_hash = &script_data[3..23];
                    let key_id = KeyId::from(
                        u160::from_slice(pubkey_hash).expect("Creating u160 from pubkey hash"),
                    );
                    let addr_str = key_id.to_string(wallet.network());
                    addresses.insert(addr_str.clone());
                    addresses.insert(format!("transparent_output:{}", addr_str));
                    output_address = addr_str;
                }
            }
            // P2SH detection
            else if script_data.len() >= 23 && script_data[0] == 0xA9 && script_data[22] == 0x87 {
                // Extract the script hash and create an address
                let script_hash = &script_data[2..22];
                let script_id = ScriptId::from(
                    u160::from_slice(script_hash).expect("Creating u160 from script hash"),
                );
                let addr_str = script_id.to_string(wallet.network());
                addresses.insert(addr_str.clone());
                addresses.insert(format!("transparent_script_output:{}", addr_str));
                output_address = addr_str;
            }

            // Check if this output is change
            if !output_address.is_empty() {
                // If this is our address and tx is from us, this is likely change
                if tx.is_from_me()
                    && wallet
                        .address_names()
                        .keys()
                        .any(|a| a.to_string() == output_address)
                {
                    // Check if this address isn't in our address book (typical of change addresses)
                    if is_likely_change_output(wallet, &output_address) {
                        is_change_transaction = true;
                        addresses.insert(format!("change_output:{}", output_address));
                    }
                }
            }

            // Track all outputs
            let output_id = format!("output:{}:{}", tx_id, vout_idx);
            addresses.insert(output_id);
        }
    }

    // Process Sapling spends and outputs with improved nullifier tracking
    if let Some(bundle) = tx.transaction().sapling_bundle() {
        for spend in bundle.shielded_spends() {
            // Track the nullifier
            let nullifier_hex: String = spend.nullifier().encode_hex();
            addresses.insert(format!("sapling_nullifier:{}", nullifier_hex));

            // If we have note data for this nullifier, find the address
            if let Some(sapling_note_data) = tx.sapling_note_data() {
                for note_data in sapling_note_data.values() {
                    if let Some(nullifier) = note_data.nullifier() {
                        if nullifier.as_slice() == spend.nullifier().as_ref() {
                            // Find the address and tag it as a spend
                            for (addr, ivk) in wallet.sapling_z_addresses() {
                                if note_data.incoming_viewing_key() == ivk {
                                    let addr_str = addr.to_string(wallet.network());
                                    addresses.insert(addr_str.clone());
                                    addresses.insert(format!("sapling_spend:{}", addr_str));
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        for output in bundle.shielded_outputs() {
            // Track the commitment
            let cm_hex = hex::encode(output.cmu().to_bytes());
            addresses.insert(format!("sapling_commitment:{}", cm_hex));

            // If we have note data for this output, find the address
            if let Some(sapling_note_data) = tx.sapling_note_data() {
                for note_data in sapling_note_data.values() {
                    for (addr, ivk) in wallet.sapling_z_addresses() {
                        if note_data.incoming_viewing_key() == ivk {
                            let addr_str = addr.to_string(wallet.network());
                            addresses.insert(addr_str.clone());
                            addresses.insert(format!("sapling_receive:{}", addr_str));
                            break;
                        }
                    }
                }
            }
        }
    }

    // Process sapling note data more thoroughly
    if let Some(sapling_note_data) = tx.sapling_note_data() {
        for (outpoint, note_data) in sapling_note_data {
            // For each note, find the corresponding address
            for (addr, ivk) in wallet.sapling_z_addresses() {
                if note_data.incoming_viewing_key() == ivk {
                    let addr_str = addr.to_string(wallet.network());
                    addresses.insert(addr_str.clone());

                    // Tag as input or output based on outpoint (outpoint is of type JSOutPoint)
                    let outpoint_str = format!("{:?}", outpoint);
                    addresses.insert(format!("sapling_note:{}", outpoint_str));

                    // If this note has a nullifier, it's been spent
                    if note_data.nullifier().is_some() {
                        addresses.insert(format!("sapling_spent_note:{}", addr_str));
                    } else {
                        addresses.insert(format!("sapling_unspent_note:{}", addr_str));
                    }
                    break;
                }
            }
        }
    }

    // Orchard action processing is done after sapling, so we don't need to process sapling note data again here

    // Improved Orchard action processing
    if let Some(orchard_bundle) = tx.transaction().orchard_bundle() {
        for (idx, action) in (0u32..).zip(orchard_bundle.actions().into_iter()) {
            let nullifier_hex = hex::encode(action.nullifier().to_bytes());
            addresses.insert(format!("orchard_nullifier:{}", nullifier_hex));

            // Track commitments
            let commitment_hex = hex::encode(action.cmx().to_bytes());
            addresses.insert(format!("orchard_commitment:{}", commitment_hex));

            // Extract additional metadata if available
            if let Some(orchard_meta) = tx.orchard_tx_meta() {
                if let Some(ivk) = orchard_meta.receiving_key(idx) {
                    // Track action by index
                    addresses.insert(format!("orchard_action:{}:{}", tx_id, idx));

                    // Instead of trying to access note data directly, just track the action
                    // Action data typically contains commitment and value information
                    addresses.insert(format!(
                        "orchard_action_data:{}",
                        hex::encode(&ivk.to_bytes()[..])
                    ));

                    // If we have recipient data from the transaction, link it
                    if let Some(recipients) = wallet.send_recipients().get(&tx_id) {
                        for recipient in recipients {
                            if let RecipientAddress::Orchard(addr) = &recipient.recipient_address {
                                addresses.insert(format!(
                                    "orchard_recipient:{}",
                                    addr.to_string(wallet.network())
                                ));
                            }
                        }
                    }
                }
            }

            // Add the action index as a unique identifier
            addresses.insert(format!("orchard_action_idx:{}:{}", tx_id, idx));
        }
    }

    // Tag transaction type
    if is_change_transaction {
        addresses.insert("transaction_type:change".to_string());
    } else if tx.is_from_me() {
        addresses.insert("transaction_type:send".to_string());
    } else {
        addresses.insert("transaction_type:receive".to_string());
    }

    // If the transaction is marked as "from me" but we don't have specific addresses
    if tx.is_from_me()
        && !addresses.iter().any(|a| {
            a.starts_with("transparent_spend:")
                || a.starts_with("sapling_spend:")
                || a.starts_with("orchard_nullifier:")
        })
    {
        // Add all our addresses as potential sources, but mark them as uncertain
        for addr in wallet.sapling_z_addresses().keys() {
            let addr_str = addr.to_string(wallet.network());
            addresses.insert(format!("possible_source:{}", addr_str));
        }

        for addr in wallet.address_names().keys() {
            let addr_str: String = addr.clone().into();
            addresses.insert(format!("possible_source:{}", addr_str));
        }
    }

    // Always add the transaction ID as an identifier
    addresses.insert(format!("tx:{}", tx_id));

    Ok(addresses)
}

/// Check if an output address is likely a change address
fn is_likely_change_output(wallet: &ZcashdWallet, address: &str) -> bool {
    // In zcashd, change addresses are typically:
    // 1. Addresses that belong to the wallet
    // 2. Not in the address book (no associated name or purpose)
    // 3. Generated from internal key paths (m/44'/x'/y'/1/...)

    // Check if the address belongs to our wallet but has no name or purpose in the address book
    let has_address = wallet
        .address_names()
        .keys()
        .any(|a| a.to_string() == address);

    if !has_address {
        return false;
    }

    // Check if this address has a name or purpose
    let address_obj = wallet
        .address_names()
        .keys()
        .find(|a| a.to_string() == address);
    if let Some(addr) = address_obj {
        // If the address has a name or purpose, it's probably not change
        if let Some(name) = wallet.address_names().get(addr) {
            if !name.is_empty() {
                return false;
            }
        }

        if let Some(purpose) = wallet.address_purposes().get(addr) {
            if !purpose.is_empty() {
                return false;
            }
        }
    }

    // If it's in our wallet but doesn't have a name or purpose, it's likely change
    true
}

/// Check if a nullifier belongs to a specific address
fn is_nullifier_for_address(_wallet: &ZcashdWallet, _nullifier_hex: &str, _address: &str) -> bool {
    // In a production implementation, this would check if the nullifier was derived
    // from notes sent to the given address. For now, this is a placeholder.
    // TODO: Implement proper nullifier-to-address mapping if needed
    false
}
