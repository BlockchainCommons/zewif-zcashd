use crate::{error::ExpectedLengths, parse, parser::prelude::*, Error, Result};
use zewif::Data;

use crate::zcashd_wallet::{CompactSize, u256};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PrivKey {
    data: Data,
    hash: u256,
}

impl PrivKey {
    pub fn data(&self) -> &Data {
        &self.data
    }

    pub fn as_slice(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn hash(&self) -> u256 {
        self.hash
    }
}

impl std::fmt::Debug for PrivKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PrivKey({:?})", self.data())
    }
}

impl AsRef<Data> for PrivKey {
    fn as_ref(&self) -> &Data {
        self.data()
    }
}

impl AsRef<[u8]> for PrivKey {
    fn as_ref(&self) -> &[u8] {
        self.data().as_ref()
    }
}

impl Parse for PrivKey {
    fn parse(p: &mut Parser) -> Result<Self> {
        let length = *parse!(p, CompactSize, "PrivKey size")?;
        if length != 214 && length != 279 {
            return Err(Error::InvalidLength {
                kind: "privkey",
                expected: ExpectedLengths::Multiple(&[214, 279]),
                actual: length,
            });
        }
        let data = parse!(p, data = length, "PrivKey")?;
        let hash = parse!(p, "PrivKey hash")?;
        Ok(Self { data, hash })
    }
}
