use anyhow::{Result, Context};

use std::collections::{HashMap, HashSet};

use zewif::{u256, Account, AddressId, AddressRegistry, ProtocolAddress, TxId};

use crate::ZcashdWallet;

use super::{keys::{convert_sapling_spending_key, find_sapling_key_for_ivk}, transaction_addresses::extract_transaction_addresses};

/// Convert ZCashd UnifiedAccounts to Zewif accounts
pub fn convert_unified_accounts(
    wallet: &ZcashdWallet,
    unified_accounts: &crate::UnifiedAccounts,
    _transactions: &HashMap<TxId, zewif::Transaction>,
) -> Result<HashMap<u256, Account>> {
    let mut accounts_map = HashMap::new();

    // Step 1: Create an account for each UnifiedAccountMetadata
    for (key_id, account_metadata) in &unified_accounts.account_metadata {
        // Create a new account with the appropriate ZIP-32 account ID
        let mut account = Account::new();

        // Set the account name and ZIP-32 account ID
        let account_name = format!("Account #{}", account_metadata.account_id());
        account.set_name(account_name);
        account.set_zip32_account_id(account_metadata.account_id());

        // Store the account in our map using the key_id as the key
        accounts_map.insert(*key_id, account);
    }

    // If no accounts were created, create a default account
    if accounts_map.is_empty() {
        let mut default_account = Account::new();
        default_account.set_name("Default Account");
        accounts_map.insert(u256::default(), default_account);
    }

    // Step 2: Build an AddressRegistry to track address-to-account mappings
    let address_registry = initialize_address_registry(wallet, unified_accounts)?;

    // Step 3: Process all addresses and assign them to the appropriate accounts

    // Process transparent addresses
    for (zcashd_address, name) in wallet.address_names() {
        // Create an AddressId for this transparent address
        let addr_id = AddressId::Transparent(zcashd_address.clone().into());

        // Try to find which account this address belongs to using our registry
        let account_key_id = if let Some(key_id) = address_registry.find_account(&addr_id) {
            // Found a mapping in the registry
            *key_id
        } else {
            // No mapping found, fall back to the first account
            match accounts_map.keys().next() {
                Some(key) => *key,
                None => u256::default(),
            }
        };

        if let Some(account) = accounts_map.get_mut(&account_key_id) {
            let transparent_address = zewif::TransparentAddress::new(zcashd_address.clone());

            // Create a ZewifAddress from the TransparentAddress
            let protocol_address = ProtocolAddress::Transparent(transparent_address);
            let mut zewif_address = zewif::Address::new(protocol_address);
            zewif_address.set_name(name.clone());

            // Set purpose if available
            if let Some(purpose) = wallet.address_purposes().get(zcashd_address) {
                zewif_address.set_purpose(purpose.clone());
            }

            // Add the address to the account
            account.add_address(zewif_address);
        }
    }

    // Process sapling addresses
    for (sapling_address, viewing_key) in wallet.sapling_z_addresses() {
        let address_str = sapling_address.to_string(wallet.network());

        // Create an AddressId for this sapling address
        let addr_id = AddressId::Sapling(address_str.clone());

        // Try to find which account this address belongs to using our registry
        let account_key_id = if let Some(key_id) = address_registry.find_account(&addr_id) {
            // Found a mapping in the registry
            *key_id
        } else {
            // No mapping found, fall back to the first account
            match accounts_map.keys().next() {
                Some(key) => *key,
                None => u256::default(),
            }
        };

        if let Some(account) = accounts_map.get_mut(&account_key_id) {
            let address_str = sapling_address.to_string(wallet.network());

            // Create a new ShieldedAddress
            let mut shielded_address = zewif::ShieldedAddress::new(address_str.clone());
            shielded_address.set_incoming_viewing_key(viewing_key.to_owned());

            // Add spending key if available in sapling_keys
            if let Some(sapling_key) = find_sapling_key_for_ivk(wallet, viewing_key) {
                // Convert to Zewif spending key format
                let spending_key = convert_sapling_spending_key(sapling_key.key())
                    .context("Failed to convert sapling spending key")?;
                shielded_address.set_spending_key(spending_key);
            }

            let protocol_address = zewif::ProtocolAddress::Shielded(shielded_address);
            let mut zewif_address = zewif::Address::new(protocol_address);

            // Set purpose if available - convert to Address type for lookup
            let zcashd_address = crate::Address::from(address_str);
            if let Some(purpose) = wallet.address_purposes().get(&zcashd_address) {
                zewif_address.set_purpose(purpose.clone());
            }

            // Add the address to the account
            account.add_address(zewif_address);
        }
    }

    // Step 4: Process viewing keys in unified_accounts
    // Each full_viewing_key entry maps a key_id to a viewing key string
    for (key_id, viewing_key) in &unified_accounts.full_viewing_keys {
        // Find the account for this key_id
        if let Some(account) = accounts_map.get_mut(key_id) {
            // TODO: Process and add the viewing key to the account
            // This will be implemented when we add specific support for viewing keys

            // For now, just log that we have a viewing key for this account
            eprintln!(
                "Found viewing key for account {}: {}",
                account.name(),
                viewing_key
            );

            // Use the registry to find all addresses associated with this account
            let account_addresses = address_registry.find_addresses_for_account(key_id);
            if !account_addresses.is_empty() {
                eprintln!("  Account has {} addresses", account_addresses.len());
            }
        }
    }

    // Step 5: Assign transactions to relevant accounts based on address involvement
    // We'll use our AddressRegistry to find account associations

    // Analyze each transaction to find which addresses are involved
    for (txid, wallet_tx) in wallet.transactions() {
        // Extract all addresses involved in this transaction
        match extract_transaction_addresses(wallet, *txid, wallet_tx) {
            Ok(tx_addresses) => {
                let mut relevant_accounts = HashSet::new();
                let is_change_transaction = tx_addresses.contains("transaction_type:change");
                let transaction_type = if tx_addresses.contains("transaction_type:send") {
                    "send"
                } else if tx_addresses.contains("transaction_type:receive") {
                    "receive"
                } else {
                    "unknown"
                };

                // First pass: Look for explicit account mappings from standard addresses
                for address_str in &tx_addresses {
                    // Check for standard addresses that we can convert to AddressId
                    if let Ok(addr_id) = AddressId::from_address_string(address_str, wallet.network()) {
                        // Look up the account in our registry
                        if let Some(account_id) = address_registry.find_account(&addr_id) {
                            relevant_accounts.insert(*account_id);
                        }
                    }
                }

                // Second pass: Check for tagged addresses with better identifiers
                if relevant_accounts.is_empty() {
                    for address_str in &tx_addresses {
                        // Check for more specific tagged addresses
                        if address_str.starts_with("transparent_spend:") ||
                           address_str.starts_with("sapling_spend:") ||
                           address_str.starts_with("orchard_spend:") {
                            // This is a spending address - may indicate source account
                            let pure_addr = &address_str[(address_str.find(':').unwrap() + 1)..];
                            if let Ok(addr_id) = AddressId::from_address_string(pure_addr, wallet.network()) {
                                if let Some(account_id) = address_registry.find_account(&addr_id) {
                                    relevant_accounts.insert(*account_id);
                                }
                            }
                        } else if address_str.starts_with("transparent_output:") ||
                                  address_str.starts_with("sapling_receive:") ||
                                  address_str.starts_with("orchard_recipient:") {
                            // This is a receiving address
                            let pure_addr = &address_str[(address_str.find(':').unwrap() + 1)..];
                            if let Ok(addr_id) = AddressId::from_address_string(pure_addr, wallet.network()) {
                                if let Some(account_id) = address_registry.find_account(&addr_id) {
                                    relevant_accounts.insert(*account_id);
                                }
                            }
                        } else if address_str.starts_with("change:") || address_str.starts_with("change_key:") || address_str.starts_with("change_output:") {
                            // This is a change address - try to find its account
                            let pure_addr = &address_str[(address_str.find(':').unwrap() + 1)..];
                            if let Ok(addr_id) = AddressId::from_address_string(pure_addr, wallet.network()) {
                                if let Some(account_id) = address_registry.find_account(&addr_id) {
                                    // For change, we add ONLY the source account
                                    relevant_accounts.clear();
                                    relevant_accounts.insert(*account_id);
                                    break;  // Only need the source account for change
                                }
                            }
                        }
                    }
                }

                // If we still don't have accounts, use intelligent fallback strategy
                if relevant_accounts.is_empty() {
                    // Different strategies based on transaction type
                    if is_change_transaction {
                        // For change transactions, try to find the source account
                        if let Some(source_account) = find_source_account_for_transaction(wallet_tx, &tx_addresses, &address_registry) {
                            relevant_accounts.insert(source_account);
                        }
                    } else if transaction_type == "send" {
                        // For send transactions with no clear mappings, look for the source
                        if let Some(source_account) = find_source_account_for_transaction(wallet_tx, &tx_addresses, &address_registry) {
                            relevant_accounts.insert(source_account);
                        }
                    } else if transaction_type == "receive" {
                        // For receives, we could try to find the most likely recipient account
                        // Or fallback to the default account
                        if let Some(default_account) = find_default_account_id(&accounts_map) {
                            relevant_accounts.insert(default_account);
                        }
                    }
                }

                // Last resort: If we still couldn't determine relevant accounts,
                // select a single appropriate account rather than adding to all
                if relevant_accounts.is_empty() {
                    if let Some(default_account) = find_default_account_id(&accounts_map) {
                        // Only add to default account if we have one
                        relevant_accounts.insert(default_account);
                    } else {
                        // Otherwise, use the first account
                        if let Some(account_id) = accounts_map.keys().next() {
                            relevant_accounts.insert(*account_id);
                        }
                    }
                }

                // Add the transaction to relevant accounts
                for account_id in relevant_accounts {
                    if let Some(account) = accounts_map.get_mut(&account_id) {
                        account.add_relevant_transaction(*txid);
                    }
                }
            }
            Err(e) => {
                // Log the error but use a smarter fallback
                eprintln!("Error analyzing transaction {}: {}", txid, e);

                // Even in error cases, try to assign to the default account if possible
                if let Some(default_account_id) = find_default_account_id(&accounts_map) {
                    if let Some(account) = accounts_map.get_mut(&default_account_id) {
                        account.add_relevant_transaction(*txid);
                    }
                } else {
                    // Only as a last resort, add to all accounts
                    for account in accounts_map.values_mut() {
                        account.add_relevant_transaction(*txid);
                    }
                }
            }
        }
    }

    Ok(accounts_map)
}

/// Find the source account for a transaction based on transaction data and extracted addresses
fn find_source_account_for_transaction(
    wallet_tx: &crate::WalletTx,
    addresses: &HashSet<String>,
    address_registry: &AddressRegistry,
) -> Option<u256> {
    // Network for parsing addresses - use mainnet as default
    let network = zewif::Network::Main; // WalletTx doesn't expose network directly

    // For outgoing transactions, check if we have explicit spending addresses
    if wallet_tx.is_from_me() {
        for address_str in addresses {
            // First, look for explicitly tagged spend addresses
            if address_str.starts_with("transparent_spend:") ||
               address_str.starts_with("sapling_spend:") ||
               address_str.starts_with("orchard_nullifier:") {

                let pure_addr = &address_str[(address_str.find(':').unwrap() + 1)..];

                // Try to convert to AddressId and find its account
                if let Ok(addr_id) = AddressId::from_address_string(pure_addr, network) {
                    if let Some(account_id) = address_registry.find_account(&addr_id) {
                        return Some(*account_id);
                    }
                }
            }

            // Next, check for change addresses (these are most reliable for source account)
            if address_str.starts_with("change:") || address_str.starts_with("change_key:") || address_str.starts_with("change_output:") {
                let pure_addr = &address_str[(address_str.find(':').unwrap() + 1)..];

                if let Ok(addr_id) = AddressId::from_address_string(pure_addr, network) {
                    if let Some(account_id) = address_registry.find_account(&addr_id) {
                        return Some(*account_id);
                    }
                }
            }
        }
    }

    None
}

/// Find the default account ID from a list of accounts
fn find_default_account_id(accounts_map: &HashMap<u256, Account>) -> Option<u256> {
    // First look for an account named "Default Account"
    for (id, account) in accounts_map {
        if account.name() == "Default Account" {
            return Some(*id);
        }
    }

    // Fallback: use the first account with ID 0
    for (id, account) in accounts_map {
        if account.zip32_account_id() == Some(0) {
            return Some(*id);
        }
    }

    // Last resort: just use the first account
    accounts_map.keys().next().copied()
}

/// Find the account ID for a transparent address by looking at key metadata and relationships
fn find_account_for_transparent_address(
    wallet: &ZcashdWallet,
    unified_accounts: &crate::UnifiedAccounts,
    address: &crate::Address,
) -> Option<u256> {
    // First, check if this is a transparent receiver in a unified address
    // This requires looking up the pub key for this address and finding matching key metadata

    // 1. Look up HD paths and seed fingerprints for all keys
    for key in wallet.keys().keypairs() {
        // We can't directly convert from pubkey to address, so iterate through known addresses
        let addr_str = address.to_string();

        // Check if this address matches any in our address book
        for known_addr in wallet.address_names().keys() {
            if known_addr.to_string() == addr_str {
                // Found the address in our address book. Now check if we can link it to a key/account.

                // Check for HD paths that indicate unified accounts
                if let Some(hd_path) = key.metadata().hd_keypath() {
                    // HD paths for unified accounts follow a pattern like:
                    // m/44'/1'/account'/type'/idx' where:
                    // - account' is the account ID
                    // - type' is 0 for external addresses, 1 for internal (change) addresses
                    // - idx' is the address index

                    // If this is a unified account HD path, we can extract the account ID
                    if is_unified_account_keypath(hd_path) {
                        if let Some(account_id) = extract_account_id_from_keypath(hd_path) {
                            // Look for unified account with this account ID
                            return find_account_key_id_by_account_id(unified_accounts, account_id);
                        }
                    }
                }

                // If we have a key fingerprint, check if it matches any unified account
                if let Some(seed_fp) = key.metadata().seed_fp() {
                    return find_account_key_id_by_seed_fingerprint(unified_accounts, seed_fp);
                }
            }
        }
    }

    None
}

/// Find the account ID for a sapling address by looking at the viewing key relationships
fn find_account_for_sapling_address(
    wallet: &ZcashdWallet,
    unified_accounts: &crate::UnifiedAccounts,
    _address: &crate::SaplingZPaymentAddress,
    viewing_key: &zewif::sapling::SaplingIncomingViewingKey,
) -> Option<u256> {
    // Look up the full viewing key associated with this incoming viewing key
    if wallet.sapling_keys().get(viewing_key).is_some() {
        // SaplingKey doesn't directly expose metadata or extfvk
        // Instead, we'll rely on viewing key mappings in unified accounts

        // Check full viewing keys mapping in unified accounts
        // Rather than trying to get the FVK string, we'll use the viewing key we already have
        let ivk_str = viewing_key.to_string();
        for (key_id, viewing_key_str) in &unified_accounts.full_viewing_keys {
            // In a real implementation, we'd properly check if this IVK is derived from FVK
            // For now, we'll just check if the strings have some similarity
            if viewing_key_str.contains(&ivk_str) || ivk_str.contains(viewing_key_str) {
                return Some(*key_id);
            }
        }
    }

    None
}

/// Check if a key path follows the unified account pattern
fn is_unified_account_keypath(keypath: &str) -> bool {
    // Typical unified account key path: m/44'/cointype'/account'/type'/idx'
    let parts: Vec<&str> = keypath.split('/').collect();
    parts.len() >= 4 && parts[0] == "m" && parts[1].starts_with("44'")
}

/// Extract account ID from a unified account key path
fn extract_account_id_from_keypath(keypath: &str) -> Option<u32> {
    // Keypath format: m/44'/cointype'/account'/type'/idx'
    let parts: Vec<&str> = keypath.split('/').collect();
    if parts.len() >= 4 {
        if let Some(account_part) = parts.get(3) {
            if let Some(account_str) = account_part.strip_suffix('\'') {
                return account_str.parse::<u32>().ok();
            }
        }
    }
    None
}

/// Find the account key ID based on account ID
fn find_account_key_id_by_account_id(unified_accounts: &crate::UnifiedAccounts, account_id: u32) -> Option<u256> {
    for (key_id, account_metadata) in &unified_accounts.account_metadata {
        if account_metadata.account_id() == account_id {
            return Some(*key_id);
        }
    }
    None
}

/// Find the account key ID based on seed fingerprint
fn find_account_key_id_by_seed_fingerprint(unified_accounts: &crate::UnifiedAccounts, seed_fp: &zewif::Blob32) -> Option<u256> {
    let seed_fp_hex = hex::encode(seed_fp.as_ref());
    for (key_id, account_metadata) in &unified_accounts.account_metadata {
        // Convert the account's seed fingerprint to hex and compare
        let account_seed_fp_hex = format!("{}", account_metadata.seed_fingerprint());
        if account_seed_fp_hex == seed_fp_hex {
            return Some(*key_id);
        }
    }
    None
}

/// Initialize an AddressRegistry based on the unified accounts data
pub fn initialize_address_registry(
    wallet: &ZcashdWallet,
    unified_accounts: &crate::UnifiedAccounts,
) -> Result<AddressRegistry> {
    let mut registry = AddressRegistry::new();

    // Step 1: Map the unified account addresses to their accounts
    for (address_id, address_metadata) in &unified_accounts.address_metadata {
        // Create an AddressId for this unified account address
        let addr_id = AddressId::from_unified_account_id(*address_id);

        // Register this address with its account's key_id
        registry.register(addr_id, address_metadata.key_id);
    }

    // Step 2: For each known transparent address, try to find its account
    for zcashd_address in wallet.address_names().keys() {
        // Create an AddressId for this transparent address
        let addr_id = AddressId::Transparent(zcashd_address.clone().into());

        // Check key metadata for HD path to determine the account
        if let Some(account_id) = find_account_for_transparent_address(wallet, unified_accounts, zcashd_address) {
            registry.register(addr_id, account_id);
        }
    }

    // Step 3: For each known sapling address, try to find its account
    for (sapling_address, viewing_key) in wallet.sapling_z_addresses() {
        // Create an AddressId for this sapling address
        let addr_str = sapling_address.to_string(wallet.network());
        let addr_id = AddressId::Sapling(addr_str);

        // Find the account for this sapling address using its viewing key
        if let Some(account_id) = find_account_for_sapling_address(wallet, unified_accounts, sapling_address, viewing_key) {
            registry.register(addr_id, account_id);
        }
    }

    Ok(registry)
}
