use zewif::sapling::SaplingIncomingViewingKey;

use crate::{zcashd_wallet::sapling::SaplingKey, ZcashdWallet};

/// Find a SaplingKey for a given incoming viewing key
pub fn find_sapling_key_for_ivk<'a>(
    wallet: &'a ZcashdWallet,
    ivk: &SaplingIncomingViewingKey,
) -> Option<&'a SaplingKey> {
    wallet.sapling_keys().get(ivk)
}
