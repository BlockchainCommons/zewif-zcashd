use std::collections::HashMap;
use zcash_keys::keys::UnifiedFullViewingKey;

use crate::zcashd_wallet::{UfvkFingerprint, UnifiedAccountMetadata, UnifiedAddressMetadata};

#[derive(Debug, Clone)]
pub struct UnifiedAccounts {
    pub address_metadata: Vec<UnifiedAddressMetadata>,
    pub full_viewing_keys: HashMap<UfvkFingerprint, UnifiedFullViewingKey>,
    pub account_metadata: HashMap<UfvkFingerprint, UnifiedAccountMetadata>,
}

impl UnifiedAccounts {
    pub fn none() -> Self {
        Self {
            address_metadata: vec![],
            full_viewing_keys: HashMap::new(),
            account_metadata: HashMap::new(),
        }
    }

    pub fn new(
        address_metadata: Vec<UnifiedAddressMetadata>,
        full_viewing_keys: HashMap<UfvkFingerprint, UnifiedFullViewingKey>,
        account_metadata: HashMap<UfvkFingerprint, UnifiedAccountMetadata>,
    ) -> Self {
        Self {
            address_metadata,
            full_viewing_keys,
            account_metadata,
        }
    }
}
