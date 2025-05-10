use anyhow::{Error, Result, bail};
use zewif::{Blob32, HexParseError};

use crate::{parse, parser::prelude::*};

pub const U256_SIZE: usize = 32;

/// A 256-bit unsigned integer represented as a 32-byte array in little-endian byte order.
///
/// This type is used throughout ZCash data structures to represent hashes, block hashes,
/// transaction IDs, and other cryptographic values that require 256 bits of precision.
///
/// # Zcash Concept Relation
/// In Zcash, many protocol elements use 256-bit values:
/// - Block hashes
/// - Transaction IDs (txids)
/// - Nullifiers
/// - Merkle tree nodes
/// - Various cryptographic commitments
///
/// The 256-bit size provides the cryptographic strength needed for secure hash representations
/// while maintaining compatibility with common cryptographic primitives like SHA-256.
///
/// # Data Preservation
/// The `u256` type preserves the exact 32-byte representation of 256-bit values found
/// in the Zcash protocol, ensuring cryptographic integrity during wallet migrations.
///
/// # Examples
/// ```
/// # use zewif_zcashd::zcashd_wallet::u256;
/// // Parse the hash of the Zcash genesis block.
/// let block_hash = u256::from_hex("00040fe8ec8471911baa1db1266ea15dd06b4a8a5c453883c000b031973dce08").unwrap();
///
/// // Display values are shown in reversed byte order (as is conventional in Bitcoin/Zcash)
/// assert_eq!(
///     format!("{}", block_hash),
///     "00040fe8ec8471911baa1db1266ea15dd06b4a8a5c453883c000b031973dce08"
/// );
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
#[allow(non_camel_case_types)]
pub struct u256([u8; U256_SIZE]);

impl u256 {
    /// Parses a `u256` value from a hexadecimal string. In conformance with zcashd, the
    /// hexadecimal representation of a `u256` is canonically in byte-reversed order.
    ///
    /// # Examples
    /// ```
    /// # use zewif_zcashd::zcashd_wallet::u256;
    /// // Parse the hash of the Zcash genesis block.
    /// let block_hash = u256::from_hex("00040fe8ec8471911baa1db1266ea15dd06b4a8a5c453883c000b031973dce08").unwrap();
    /// ```
    pub fn from_hex(hex: &str) -> Result<Self, HexParseError> {
        let blob = Blob32::from_hex(hex)?;
        let mut bytes = <[u8; U256_SIZE]>::from(blob);
        bytes.reverse();
        Ok(Self(bytes))
    }

    pub fn into_bytes(self) -> [u8; U256_SIZE] {
        self.0
    }
}

impl TryFrom<&[u8]> for u256 {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != U256_SIZE {
            bail!("Invalid data length: expected 32, got {}", bytes.len());
        }
        let mut a = [0u8; U256_SIZE];
        a.copy_from_slice(bytes);
        Ok(Self(a))
    }
}

impl TryFrom<&[u8; U256_SIZE]> for u256 {
    type Error = Error;

    fn try_from(bytes: &[u8; U256_SIZE]) -> Result<Self, Self::Error> {
        Ok(Self(*bytes))
    }
}

impl TryFrom<&Vec<u8>> for u256 {
    type Error = Error;

    fn try_from(bytes: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(bytes.as_slice())
    }
}

impl AsRef<[u8]> for u256 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8; U256_SIZE]> for u256 {
    fn as_ref(&self) -> &[u8; U256_SIZE] {
        &self.0
    }
}

impl std::fmt::Debug for u256 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut bytes = self.0;
        bytes.reverse();
        write!(f, "u256({})", hex::encode(bytes))
    }
}

impl std::fmt::Display for u256 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut bytes = self.0;
        bytes.reverse();
        write!(f, "{}", hex::encode(bytes))
    }
}

impl Parse for u256 {
    fn parse(p: &mut Parser) -> Result<Self> {
        let bytes = parse!(p, "u256")?;
        Ok(Self(bytes))
    }
}
