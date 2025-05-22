use anyhow::Result;

use zewif::{Blob32, NoQuotesDebugOption};

use crate::{parse, parser::prelude::*, zcashd_wallet::SecondsSinceEpoch};

const VERSION_WITH_HDDATA: i32 = 10;

#[derive(Clone, PartialEq)]
pub struct KeyMetadata {
    version: i32,
    create_time: Option<SecondsSinceEpoch>,
    hd_keypath: Option<String>,
    seed_fp: Option<Blob32>,
}

impl std::fmt::Debug for KeyMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyMetadata")
            .field("version", &self.version)
            .field("create_time", &NoQuotesDebugOption(&self.create_time))
            .field("hd_keypath", &NoQuotesDebugOption(&self.hd_keypath))
            .field("seed_fp", &NoQuotesDebugOption(&self.seed_fp))
            .finish()
    }
}

impl KeyMetadata {
    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn create_time(&self) -> Option<SecondsSinceEpoch> {
        self.create_time
    }

    pub fn hd_keypath(&self) -> Option<&String> {
        self.hd_keypath.as_ref()
    }

    pub fn seed_fp(&self) -> Option<&Blob32> {
        self.seed_fp.as_ref()
    }
}

impl Parse for KeyMetadata {
    fn parse(p: &mut Parser) -> Result<Self> {
        let version = parse!(p, "version")?;
        let create_time: SecondsSinceEpoch = parse!(p, "create_time")?;
        // 0 means unknown (per `walletdb.h`)
        let create_time = if create_time.is_zero() {
            None
        } else {
            Some(create_time)
        };
        let hd_keypath: Option<String>;
        let seed_fp: Option<Blob32>;
        if version >= VERSION_WITH_HDDATA {
            hd_keypath = Some(parse!(p, "hd_keypath")?);
            seed_fp = Some(parse!(p, "seed_fp")?);
        } else {
            hd_keypath = None;
            seed_fp = None;
        }
        Ok(Self {
            version,
            create_time,
            hd_keypath: hd_keypath.filter(|p| !p.trim().is_empty()),
            seed_fp: seed_fp.filter(|fp| fp.as_bytes() != &[0u8; 32]),
        })
    }
}
