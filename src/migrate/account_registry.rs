use std::collections::HashMap;

use zewif::Account;

use crate::zcashd_wallet::UfvkFingerprint;

struct AccountRegistry {
    accounts: Vec<Account>,
    key_index: HashMap<UfvkFingerprint, usize>,
}

impl AccountRegistry {
    pub fn empty() -> Self {
        AccountRegistry {
            accounts: vec![],
            key_index: HashMap::new(),
        }
    }
}
