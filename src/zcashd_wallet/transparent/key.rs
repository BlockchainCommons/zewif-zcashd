use anyhow::{Result, bail};
use sha2::{Digest, Sha256};
use zewif::Data;

use crate::zcashd_wallet::{
    transparent::{PrivKey, PubKey},
    u256, KeyMetadata,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Key {
    pubkey: PubKey,
    privkey: PrivKey,
    metadata: KeyMetadata,
}

impl Key {
    pub fn pubkey(&self) -> &PubKey {
        &self.pubkey
    }

    pub fn privkey(&self) -> &PrivKey {
        &self.privkey
    }

    pub fn metadata(&self) -> &KeyMetadata {
        &self.metadata
    }
}

impl Key {
    pub fn new(pubkey: PubKey, privkey: PrivKey, metadata: KeyMetadata) -> Result<Self> {
        let hash = hash256(Data::concat(&[&pubkey, &privkey]));
        if hash != privkey.hash() {
            bail!("Invalid keypair: pubkey and privkey do not match");
        }
        Ok(Self {
            pubkey,
            privkey,
            metadata,
        })
    }
}

/// Computes a single SHA-256 hash of the provided data, returning a 256-bit result.
///
/// This function provides a standardized implementation of the SHA-256 hashing algorithm
/// used throughout Zcash protocols for various cryptographic operations, including transaction
/// identification, block hashing, and signature validation.
///
/// # Zcash Concept Relation
/// In Zcash (and other cryptocurrencies):
///
/// - **Transaction IDs**: Generated using various hashing schemes, often involving SHA-256
/// - **Merkle Trees**: Used for efficient verification of transaction inclusion
/// - **Block Headers**: Hash-chained together using SHA-256 based functions
/// - **Address Generation**: Involves hashing of public keys and other components
///
/// # Arguments
/// * `data` - The data to hash, which can be any type that implements `AsRef<[u8]>`,
///   such as `&[u8]`, `Vec<u8>`, or `String`
///
/// # Returns
/// A `u256` containing the 32-byte hash result
fn sha256(data: impl AsRef<[u8]>) -> u256 {
    let mut hasher = Sha256::new();
    hasher.update(data);
    u256::try_from(hasher.finalize().as_slice()).unwrap()
}

/// Computes a double SHA-256 hash (SHA-256 applied twice) of the provided data.
///
/// This function applies the SHA-256 algorithm twice: first to the input data,
/// then to the result of the first hash. This double-hashing approach is derived
/// from Bitcoin and is used in Zcash's transparent transaction components to maintain
/// compatibility with Bitcoin's hashing model.
///
/// # Zcash Concept Relation
/// In Zcash's transparent protocol components:
///
/// - **Transaction IDs**: Computed using double SHA-256 of the serialized transaction data
/// - **Block Headers**: Include a double SHA-256 hash of the previous block header
/// - **Merkle Roots**: Constructed using double SHA-256 for each tree level
///
/// Double hashing provides enhanced security against length-extension attacks
/// that can affect single-round SHA-256.
///
/// # Arguments
/// * `data` - The data to hash, which can be any type that implements `AsRef<[u8]>`,
///   such as `&[u8]`, `Vec<u8>`, or `String`
///
/// # Returns
/// A `u256` containing the 32-byte double hash result
fn hash256(data: impl AsRef<[u8]>) -> u256 {
    sha256(sha256(data))
}
