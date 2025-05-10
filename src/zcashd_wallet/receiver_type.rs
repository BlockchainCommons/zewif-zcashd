use anyhow::{Result, bail};

use crate::{parse, parser::prelude::*, zcashd_wallet::CompactSize};

/// ZCash receiver types used in Unified Addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ReceiverType {
    /// P2PKH (Pay to Public Key Hash) transparent address type
    P2PKH = 0x00,
    /// P2SH (Pay to Script Hash) transparent address type
    P2SH = 0x01,
    /// Sapling shielded address type
    Sapling = 0x02,
    /// Orchard shielded address type
    Orchard = 0x03,
}

/// Parses a ReceiverType from a binary data stream as encoded in zcashd's wallet.dat format.
impl Parse for ReceiverType {
    fn parse(p: &mut Parser) -> Result<Self> {
        let byte = *parse!(p, CompactSize, "ReceiverType")?;
        match byte {
            0x00 => Ok(ReceiverType::P2PKH),
            0x01 => Ok(ReceiverType::P2SH),
            0x02 => Ok(ReceiverType::Sapling),
            0x03 => Ok(ReceiverType::Orchard),
            _ => Err(anyhow::anyhow!("Invalid ReceiverType byte: 0x{:02x}", byte)),
        }
    }
}

impl From<ReceiverType> for String {
    fn from(value: ReceiverType) -> Self {
        match value {
            ReceiverType::P2PKH => "P2PKH".to_string(),
            ReceiverType::P2SH => "P2SH".to_string(),
            ReceiverType::Sapling => "Sapling".to_string(),
            ReceiverType::Orchard => "Orchard".to_string(),
        }
    }
}

impl TryFrom<String> for ReceiverType {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        match value.as_str() {
            "P2PKH" => Ok(ReceiverType::P2PKH),
            "P2SH" => Ok(ReceiverType::P2SH),
            "Sapling" => Ok(ReceiverType::Sapling),
            "Orchard" => Ok(ReceiverType::Orchard),
            _ => bail!("Invalid ReceiverType string: {}", value),
        }
    }
}
