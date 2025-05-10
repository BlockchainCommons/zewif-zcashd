use crate::zcashd_wallet::{KeyMetadata, u252};

#[derive(Debug, Clone, PartialEq)]
pub struct SproutSpendingKey {
    key: u252,
    metadata: KeyMetadata,
}

impl SproutSpendingKey {
    pub fn key(&self) -> u252 {
        self.key
    }

    pub fn metadata(&self) -> &KeyMetadata {
        &self.metadata
    }
}

impl SproutSpendingKey {
    pub fn new(key: u252, metadata: KeyMetadata) -> Self {
        Self { key, metadata }
    }
}
