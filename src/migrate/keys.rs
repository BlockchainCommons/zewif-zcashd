use anyhow::Result;

use zewif::sapling::SaplingIncomingViewingKey;

use crate::ZcashdWallet;


/// Find a SaplingKey for a given incoming viewing key
pub fn find_sapling_key_for_ivk<'a>(
    wallet: &'a ZcashdWallet,
    ivk: &SaplingIncomingViewingKey,
) -> Option<&'a crate::SaplingKey> {
    wallet.sapling_keys().get(ivk)
}

/// Convert ZCashd SaplingExtendedSpendingKey to Zewif SpendingKey
pub fn convert_sapling_spending_key(
    key: &zewif::sapling::SaplingExtendedSpendingKey,
) -> Result<zewif::SpendingKey> {
    // Create a SaplingSpendingKey with all components including HD parameters
    let sapling_key = zewif::sapling::SaplingSpendingKey::new_extended(
        key.expsk.ask,
        key.expsk.nsk,
        key.expsk.ovk,
        key.depth,
        key.parent_fvk_tag,
        key.child_index,
        key.chain_code,
        key.dk,
    );

    // Create the SpendingKey wrapper
    let spending_key = zewif::SpendingKey::Sapling(sapling_key);

    Ok(spending_key)
}
