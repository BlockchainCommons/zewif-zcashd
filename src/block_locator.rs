use anyhow::Result;

use zewif::u256;
use zewif::{parse, parser::prelude::*};

use super::ClientVersion;

/// Vector of block hashes
#[derive(Debug, Clone, PartialEq)]
pub struct BlockLocator {
    version: ClientVersion,
    blocks: Vec<u256>,
}

impl BlockLocator {
    pub fn version(&self) -> ClientVersion {
        self.version
    }

    pub fn blocks(&self) -> &[u256] {
        &self.blocks
    }
}

impl Parse for BlockLocator {
    fn parse(p: &mut Parser) -> Result<Self> {
        Ok(Self {
            version: parse!(p, "version")?,
            blocks: parse!(p, "blocks")?,
        })
    }
}
