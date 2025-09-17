use crate::ZcashdWallet;
use crate::Result;

/// Convert ZCashd mnemonic seed to Zewif SeedMaterial
pub fn convert_seed_material(wallet: &ZcashdWallet) -> Result<Option<zewif::SeedMaterial>> {
    // Check if we have a mnemonic phrase
    if !wallet.bip39_mnemonic().mnemonic().is_empty() {
        return Ok(Some(zewif::SeedMaterial::Bip39Mnemonic(
            wallet.bip39_mnemonic().clone(),
        )));
    }
    // If no mnemonic, return None
    Ok(None)
}
