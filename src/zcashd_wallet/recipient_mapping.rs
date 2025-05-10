use crate::zcashd_wallet::RecipientAddress;

#[derive(Debug, Clone, PartialEq)]
pub struct RecipientMapping {
    pub recipient_address: RecipientAddress,
    pub unified_address: String,
}

impl RecipientMapping {
    pub fn new(recipient_address: RecipientAddress, unified_address: String) -> Self {
        Self {
            recipient_address,
            unified_address,
        }
    }
}
