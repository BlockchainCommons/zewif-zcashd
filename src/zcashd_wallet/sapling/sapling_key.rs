use anyhow::Result;

use zewif::sapling::SaplingIncomingViewingKey;

use crate::zcashd_wallet::KeyMetadata;


#[derive(Debug, Clone, PartialEq)]
pub struct SaplingKey {
    ivk: SaplingIncomingViewingKey,
    extsk: sapling::zip32::ExtendedSpendingKey,
    metadata: KeyMetadata,
}

impl SaplingKey {
    pub fn new(
        ivk: SaplingIncomingViewingKey,
        extsk: sapling::zip32::ExtendedSpendingKey,
        metadata: KeyMetadata,
    ) -> Result<Self> {
        Ok(Self { ivk, extsk, metadata })
    }

    pub fn ivk(&self) -> &SaplingIncomingViewingKey {
        &self.ivk
    }

    pub fn extsk(&self) -> &sapling::zip32::ExtendedSpendingKey {
        &self.extsk
    }

    pub fn metadata(&self) -> &KeyMetadata {
        &self.metadata
    }
}
