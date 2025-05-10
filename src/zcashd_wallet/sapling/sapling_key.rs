use anyhow::Result;

use zewif::sapling::SaplingIncomingViewingKey;

use crate::zcashd_wallet::KeyMetadata;


#[derive(Debug, Clone, PartialEq)]
pub struct SaplingKey {
    ivk: SaplingIncomingViewingKey,
    key: sapling::zip32::ExtendedSpendingKey,
    metadata: KeyMetadata,
}

impl SaplingKey {
    pub fn new(
        ivk: SaplingIncomingViewingKey,
        key: sapling::zip32::ExtendedSpendingKey,
        metadata: KeyMetadata,
    ) -> Result<Self> {
        Ok(Self { ivk, key, metadata })
    }

    pub fn ivk(&self) -> &SaplingIncomingViewingKey {
        &self.ivk
    }

    pub fn key(&self) -> &sapling::zip32::ExtendedSpendingKey {
        &self.key
    }

    pub fn metadata(&self) -> &KeyMetadata {
        &self.metadata
    }
}
