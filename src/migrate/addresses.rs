use anyhow::{Result, anyhow};
use zcash_keys::keys::UnifiedAddressRequest;
use zip32::DiversifierIndex;

use std::collections::HashMap;

use zewif::{Account, ProtocolAddress, UnifiedAddress, sapling::SaplingExtendedSpendingKey};

use super::keys::find_sapling_key_for_ivk;
use crate::{
    ZcashdWallet,
    migrate::{AddressId, AddressRegistry},
    zcashd_wallet::{Address, ReceiverType, UfvkFingerprint},
};

/// Convert ZCashd transparent addresses to Zewif format
///
/// This function handles transparent address assignment:
/// - If registry is available, tries to map addresses to accounts
/// - Otherwise assigns all addresses to the default account
pub fn convert_transparent_addresses(
    wallet: &ZcashdWallet,
    default_account: &mut zewif::Account,
    address_registry: Option<&AddressRegistry>,
    accounts_map: &mut Option<&mut HashMap<UfvkFingerprint, Account>>,
) -> Result<()> {
    // Flag for multi-account mode
    let multi_account_mode = address_registry.is_some() && accounts_map.is_some();

    // Process address_names which contain transparent addresses
    for (zcashd_address, name) in wallet.address_names() {
        // Create address components
        let transparent_address = zewif::transparent::Address::new(zcashd_address.clone());
        let protocol_address = ProtocolAddress::Transparent(transparent_address);
        let mut zewif_address = zewif::Address::new(protocol_address);
        zewif_address.set_name(name.clone());

        // Set purpose if available
        if let Some(purpose) = wallet.address_purposes().get(zcashd_address) {
            zewif_address.set_purpose(purpose.clone());
        }

        // In multi-account mode, try to assign to the correct account
        let mut assigned = false;

        if multi_account_mode {
            let registry = address_registry.unwrap();
            let addr_id = AddressId::Transparent(zcashd_address.clone().into());

            if let Some(account_id) = registry.find_account(&addr_id) {
                if let Some(accounts) = accounts_map.as_mut() {
                    if let Some(target_account) = accounts.get_mut(account_id) {
                        // Add to the specified account
                        target_account.add_address(zewif_address.clone());
                        assigned = true;
                    }
                }
            }
        }

        // If not assigned to an account or in single-account mode, add to default account
        if !assigned {
            default_account.add_address(zewif_address);
        }
    }

    Ok(())
}

/// Convert ZCashd sapling addresses to Zewif format
///
/// This function handles sapling address assignment:
/// - If registry is available, tries to map addresses to accounts
/// - Otherwise assigns all addresses to the default account
pub fn convert_sapling_addresses(
    wallet: &ZcashdWallet,
    default_account: &mut zewif::Account,
    address_registry: Option<&AddressRegistry>,
    accounts_map: &mut Option<&mut HashMap<UfvkFingerprint, Account>>,
) -> Result<()> {
    // Flag for multi-account mode
    let multi_account_mode = address_registry.is_some() && accounts_map.is_some();

    // Process sapling_z_addresses
    for (sapling_address, viewing_key) in wallet.sapling_z_addresses() {
        let address_str = sapling_address.to_string(wallet.network());

        // Create a new ShieldedAddress and preserve the incoming viewing key
        // This is critical for maintaining the ability to detect incoming transactions
        // Note: We preserve IVKs but not FVKs, as FVKs can be derived from spending keys when needed
        let mut shielded_address = zewif::sapling::Address::new(address_str.clone());
        shielded_address.set_incoming_viewing_key(viewing_key.to_owned()); // Preserve the IVK exactly as in source wallet

        // Add spending key if available in sapling_keys
        if let Some(sapling_key) = find_sapling_key_for_ivk(wallet, viewing_key) {
            shielded_address.set_spending_key(SaplingExtendedSpendingKey::new(
                sapling_key.key().to_bytes(),
            ));
        }

        let protocol_address = zewif::ProtocolAddress::Sapling(Box::new(shielded_address));
        let mut zewif_address = zewif::Address::new(protocol_address);

        // Set purpose if available - convert to Address type for lookup
        let zcashd_address = Address::from(address_str.clone());
        if let Some(purpose) = wallet.address_purposes().get(&zcashd_address) {
            zewif_address.set_purpose(purpose.clone());
        }

        // In multi-account mode, try to assign to the correct account
        let mut assigned = false;

        if multi_account_mode {
            let registry = address_registry.unwrap();
            let addr_id = AddressId::Sapling(address_str.clone());

            if let Some(account_id) = registry.find_account(&addr_id) {
                if let Some(accounts) = accounts_map.as_mut() {
                    if let Some(target_account) = accounts.get_mut(account_id) {
                        // Add to the specified account
                        target_account.add_address(zewif_address.clone());
                        assigned = true;
                    }
                }
            }
        }

        // If not assigned to an account or in single-account mode, add to default account
        if !assigned {
            default_account.add_address(zewif_address);
        }
    }

    Ok(())
}

/// Convert ZCashd unified addresses to Zewif format
///
/// This function handles unified address extraction and assignment:
/// - Extracts unified addresses from UnifiedAddressMetadata
/// - Preserves diversifier indices and receiver types
/// - Assigns unified addresses to appropriate accounts using the registry
pub fn convert_unified_addresses(
    wallet: &ZcashdWallet,
    default_account: &mut zewif::Account,
    address_registry: Option<&AddressRegistry>,
    accounts_map: &mut Option<&mut HashMap<UfvkFingerprint, Account>>,
) -> Result<()> {
    // Only process if we have unified accounts
    let unified_accounts = wallet.unified_accounts();

    // Multi-account mode is active when we have both a registry and accounts map
    // TODO: figure out why this is being checked
    let multi_account_mode = address_registry.is_some() && accounts_map.is_some();

    // Process unified address metadata entries
    for metadata in &unified_accounts.address_metadata {
        let account = unified_accounts.account_metadata.get(&metadata.key_id);
        let ufvk = unified_accounts
            .full_viewing_keys
            .get(&metadata.key_id)
            .ok_or(anyhow!(
                "No UFVK was found for UFVK fingerprint {}",
                metadata.key_id.to_hex()
            ))?;

        let ua_str = {
            let j = DiversifierIndex::from(<[u8; 11]>::from(metadata.diversifier_index.clone()));
            let request = UnifiedAddressRequest::new(
                metadata.receiver_types.contains(&ReceiverType::P2PKH),
                metadata.receiver_types.contains(&ReceiverType::Sapling),
                metadata.receiver_types.contains(&ReceiverType::Orchard),
            )
            .ok_or(anyhow!(
                "Receiver types do not produce a valid Unified address."
            ))?;

            ufvk.address(j, request)?
                .encode(&wallet.network_info().to_address_encoding_network())
        };

        // Construct the unified address with its derivation metadata.
        let unified_address = UnifiedAddress::from_parts(
            ua_str.clone(),
            Some(metadata.diversifier_index.clone()),
            account.map(|a| format!("m/32'/{}'/{}'", a.bip_44_coin_type(), a.zip32_account_id())),
        );

        // Try to find transparent and sapling components for this unified address
        // from already processed addresses in the wallet

        // Create a unified address protocol address
        let zewif_address =
            zewif::Address::new(ProtocolAddress::Unified(Box::new(unified_address)));

        // Set purpose if available - though we may not have explicit purposes for unified addresses
        // in current wallet structure, this is here for future compatibility

        // In multi-account mode, try to assign to the correct account
        let mut assigned = false;

        if multi_account_mode {
            let registry = address_registry.unwrap();
            let addr_id = AddressId::Unified(ua_str[0..20].to_string());

            if let Some(account_id) = registry.find_account(&addr_id) {
                if let Some(accounts) = accounts_map.as_mut() {
                    if let Some(target_account) = accounts.get_mut(account_id) {
                        // Add to the specified account
                        target_account.add_address(zewif_address.clone());
                        assigned = true;
                    }
                }
            } else {
                // Try with the Unified variant if UnifiedAccountAddress didn't work
                let addr_id = AddressId::Unified(ua_str);
                if let Some(account_id) = registry.find_account(&addr_id) {
                    if let Some(accounts) = accounts_map.as_mut() {
                        if let Some(target_account) = accounts.get_mut(account_id) {
                            // Add to the specified account
                            target_account.add_address(zewif_address.clone());
                            assigned = true;
                        }
                    }
                }
            }
        }

        // If not assigned to an account or in single-account mode, add to default account
        if !assigned {
            default_account.add_address(zewif_address);
        }
    }

    Ok(())
}
