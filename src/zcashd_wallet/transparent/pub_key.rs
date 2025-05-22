use anyhow::{Context, Result, bail};

use zewif::Data;

use crate::{parse, parser::prelude::*, zcashd_wallet::CompactSize};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PubKey(Data);

impl PubKey {
    pub const PUBLIC_KEY_SIZE: usize = 65;
    pub const COMPRESSED_PUBLIC_KEY_SIZE: usize = 33;

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_ref()
    }

    pub fn is_compressed(&self) -> bool {
        self.0.as_slice().len() == Self::COMPRESSED_PUBLIC_KEY_SIZE
    }
}

impl std::fmt::Debug for PubKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PubKey({:?})", &self.0)
    }
}

impl AsRef<Data> for PubKey {
    fn as_ref(&self) -> &Data {
        &self.0
    }
}

impl AsRef<[u8]> for PubKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Parse for PubKey {
    fn parse(p: &mut Parser) -> Result<Self> {
        let size = *parse!(p, CompactSize, "PubKey size")?;
        if size != 33 && size != 65 {
            bail!("Invalid PubKey size: {}", size);
        }

        let key_data = p.next(size).map(Data::from_slice).context("PubKey")?;
        Ok(Self(key_data))
    }
}
