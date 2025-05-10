use std::collections::HashMap;

use anyhow::Result;

use crate::{ZcashdWallet, zcashd_wallet::UfvkFingerprint};

use zewif::{self, Account, BlockHeight, TxId, Zewif, ZewifWallet};

use super::{
    convert_sapling_addresses, convert_seed_material, convert_transactions,
    convert_transparent_addresses, convert_unified_accounts, convert_unified_addresses,
    initialize_address_registry,
};

/// Migrate a ZCashd wallet to the Zewif wallet format
pub fn migrate_to_zewif(wallet: &ZcashdWallet, export_height: BlockHeight) -> Result<Zewif> {
    // Create a new Zewif
    let mut zewif = Zewif::new(export_height);

    // Convert seed material (mnemonic phrase)
    let seed_material = convert_seed_material(wallet)?;

    // Create a complete Zewif wallet
    let mut zewif_wallet = ZewifWallet::new(wallet.network());

    if let Some(seed_material) = seed_material {
        zewif_wallet.set_seed_material(seed_material);
    }

    // Process transactions and collect relevant transaction IDs
    let mut transactions = convert_transactions(wallet)?;

    // For each of our received transactions, record the most stable witness.
    set_received_output_witnesses(wallet, &mut transactions)?;

    // Add an account to the wallet for each unified account
    let mut accounts_map = {
        let unified_accounts = wallet.unified_accounts();

        // Create accounts based on unified_accounts structure
        let mut accounts_map = convert_unified_accounts(wallet, unified_accounts, &transactions)?;

        // Initialize address registry to track address-to-account relationships
        let address_registry = initialize_address_registry(wallet, unified_accounts)?;

        // Create a default account for addresses not associated with any other account
        let mut default_account = Account::new();
        default_account.set_name("Default Account");

        // Create a mutable reference for accounts_map to use in the conversion functions
        {
            let mut accounts_map_ref = Some(&mut accounts_map);

            // Convert transparent addresses using the registry to assign to correct accounts
            convert_transparent_addresses(
                wallet,
                &mut default_account,
                Some(&address_registry),
                &mut accounts_map_ref,
            )?;

            // Convert sapling addresses using the registry to assign to correct accounts
            convert_sapling_addresses(
                wallet,
                &mut default_account,
                Some(&address_registry),
                &mut accounts_map_ref,
            )?;

            // Convert unified addresses using the registry to assign to correct accounts
            convert_unified_addresses(
                wallet,
                &mut default_account,
                Some(&address_registry),
                &mut accounts_map_ref,
            )?;
        }

        // Add the default account to accounts_map if it has any addresses
        if !default_account.addresses().is_empty() {
            // FIXME: the accounts map should be a secondary index for fast lookups, not primary
            // storage.
            accounts_map.insert(UfvkFingerprint::new([0u8; 32]), default_account);
        }

        accounts_map
    };

    // Add an account to the wallet for each legacy pool of funds.
    {
        // FIXME: Add the legacy account and any other accounts (legacy Sapling keys allocated via
        // `z_getnewaddress`, imported transparent accounts, etc.)
        for account in accounts_map.values() {
            zewif_wallet.add_account(account.clone());
        }

        // No unified accounts - create a single default account
        let mut default_account = Account::new();
        default_account.set_name("Default Account");

        // Create a None reference for accounts_map
        let mut accounts_map_ref = Some(&mut accounts_map);

        // Convert transparent addresses (single account mode)
        convert_transparent_addresses(wallet, &mut default_account, None, &mut accounts_map_ref)?;

        // Convert sapling addresses (single account mode)
        convert_sapling_addresses(wallet, &mut default_account, None, &mut accounts_map_ref)?;

        // Add all transaction IDs to the default account's relevant transactions
        for txid in transactions.keys() {
            default_account.add_relevant_transaction(*txid);
        }

        // Add the default account to the wallet
        zewif_wallet.add_account(default_account);
    }

    // Add wallet and transactions to the Zewif
    zewif.add_wallet(zewif_wallet);
    zewif.set_transactions(transactions);

    Ok(zewif)
}

/// Update transaction outputs with note positions from the note commitment tree
fn set_received_output_witnesses(
    wallet: &ZcashdWallet,
    _transactions: &mut HashMap<TxId, zewif::Transaction>,
) -> Result<()> {
    // Get the orchard note commitment tree from the wallet
    let _note_commitment_tree = wallet.orchard_note_commitment_tree();

    // For each transaction output belonging to the wallet, store the witness at the stable height
    // (100 blocks from the chain tip) if available. Do not store any witnesses more recent than
    // the stable height; the wallet will need to re-scan the last 100 blocks on import of a ZeWIF
    // export.
    todo!()
    //for (_tx_id, tx) in transactions.iter_mut() {
    //    // Get mutable access to the transaction components

    //    // Update Orchard actions with positions
    //    let orchard_actions = tx.orchard_actions_mut();
    //    if let Some(actions) = orchard_actions {
    //        for action in actions {
    //            let commitment = action.commitment();
    //            if let Some(position) = commitment_positions.get(commitment) {
    //                action.set_note_commitment_tree_position(*position);
    //            }
    //        }
    //    }

    //    // Update Sapling outputs with positions
    //    let sapling_outputs = tx.sapling_outputs_mut();
    //    if let Some(outputs) = sapling_outputs {
    //        for output in outputs {
    //            let commitment = output.commitment();
    //            if let Some(position) = commitment_positions.get(commitment) {
    //                output.set_note_commitment_tree_position(*position);
    //            }
    //        }
    //    }
    //}

    //Ok(())
}
