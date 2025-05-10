use anyhow::{Context, Result};
use zewif::SeedFingerprint;

use crate::{parse, parser::prelude::*};

/// This s a zcashd-specific internal unique identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UfvkFingerprint([u8; 32]);

impl UfvkFingerprint {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn from_bytes(xs: &[u8]) -> Result<Self> {
        let id_bytes = <[u8; 32]>::try_from(xs)
            .context("Failed to parse UFVK fingerprint from unified account ID bytes")?;
        Ok(Self(id_bytes))
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl Parse for UfvkFingerprint {
    fn parse(p: &mut Parser) -> Result<Self> {
        let bytes = parse!(p, "ufvk_fingerprint")?;
        Ok(Self(bytes))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnifiedAccountMetadata {
    seed_fingerprint: SeedFingerprint,
    ufvk_fingerprint: UfvkFingerprint,
    bip_44_coin_type: u32,
    zip32_account_id: u32,
}

impl UnifiedAccountMetadata {
    pub fn seed_fingerprint(&self) -> &SeedFingerprint {
        &self.seed_fingerprint
    }

    pub fn ufvk_fingerprint(&self) -> &UfvkFingerprint {
        &self.ufvk_fingerprint
    }

    pub fn bip_44_coin_type(&self) -> u32 {
        self.bip_44_coin_type
    }

    pub fn zip32_account_id(&self) -> u32 {
        self.zip32_account_id
    }
}

impl Parse for UnifiedAccountMetadata {
    fn parse(p: &mut Parser) -> Result<Self> {
        let seed_fingerprint = parse!(p, "seed_fingerprint")?;
        let bip_44_coin_type = parse!(p, "bip_44_coin_type")?;
        let zip32_account_id = parse!(p, "account_id")?;
        let ufvk_fingerprint = parse!(p, "key_id")?;
        Ok(Self {
            seed_fingerprint,
            ufvk_fingerprint,
            bip_44_coin_type,
            zip32_account_id,
        })
    }
}
