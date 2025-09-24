use std::{borrow::Cow, error::Error as StdError, fmt, io};

use thiserror::Error;

/// Crate-local error type for zewif-zcashd.
///
/// This mirrors the structure adopted in the `zewif` crate and gives us
/// a single place to describe the various failure modes we encounter while
/// decoding `zcashd` wallet data. Most conversions rely on `#[from]` to keep
/// `?` ergonomics intact; ad-hoc failures can use the `Message` variant.
#[derive(Debug, Error)]
pub enum Error {
    /// Wraps an arbitrary source error with additional static context.
    #[error("{message}")]
    Context {
        message: Cow<'static, str>,
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },

    /// IO failures from filesystem and process interactions.
    #[error(transparent)]
    Io(#[from] io::Error),

    /// Hex decoding problems when parsing serialized blobs.
    #[error(transparent)]
    Hex(#[from] hex::FromHexError),

    /// Errors bubbled up from the core `zewif` crate.
    #[error(transparent)]
    Zewif(#[from] zewif::Error),

    /// Unified address parsing errors from `zcash_address`.
    #[error(transparent)]
    UnifiedAddressParse(#[from] zcash_address::unified::ParseError),

    /// Unified full viewing key decoding errors from `zcash_keys`.
    #[error(transparent)]
    UfvkDecoding(#[from] zcash_keys::keys::DecodingError),

    /// Unified address generation failures from `zcash_keys`.
    #[error(transparent)]
    AddressGeneration(#[from] zcash_keys::keys::AddressGenerationError),

    /// Parser consumed less data than expected.
    #[error("buffer not fully consumed: {remaining} bytes remain")]
    BufferNotConsumed { remaining: usize },

    /// Parser attempted to consume more data than remains.
    #[error(
        "buffer underflow at offset {offset}: needed {needed} bytes but only {remaining} remaining"
    )]
    BufferUnderflow {
        offset: usize,
        needed: usize,
        remaining: usize,
    },

    /// Missing record while parsing wallet data.
    #[error("missing {kind}: {key}")]
    MissingRecord { kind: &'static str, key: String },

    /// Duplicate record encountered while parsing wallet data.
    #[error("duplicate {kind}: {key}")]
    DuplicateRecord { kind: &'static str, key: String },

    /// Unexpected number of records for a key.
    #[error(
        "{kind} expected exactly one record for {identifier}, found {count}"
    )]
    UnexpectedRecordCount {
        kind: &'static str,
        identifier: String,
        count: usize,
    },

    /// Failure running an external command.
    #[error("{command} failed: {message}")]
    CommandFailure {
        command: &'static str,
        status: Option<i32>,
        message: String,
    },

    /// Inconsistencies detected in the Berkeley DB dump.
    #[error("inconsistent Berkeley DB dump: {reason}")]
    DumpInconsistency { reason: DumpInconsistency },

    /// Boolean value outside the accepted range.
    #[error("invalid boolean value: {value}")]
    InvalidBoolean { value: u8 },

    /// Optional value discriminant was not recognised.
    #[error("invalid optional discriminant: 0x{value:02x}")]
    InvalidOptionalDiscriminant { value: u8 },

    /// Data length did not match expectations.
    #[error("invalid length for {kind}: expected {expected}, got {actual}")]
    InvalidLength {
        kind: &'static str,
        expected: ExpectedLengths,
        actual: usize,
    },

    /// Numeric amount is outside acceptable bounds.
    #[error("invalid {kind} amount: {value}")]
    InvalidAmount { kind: &'static str, value: i64 },

    /// Receiver type byte is unknown.
    #[error("invalid receiver type byte: 0x{byte:02x}")]
    InvalidReceiverTypeByte { byte: usize },

    /// Receiver type string is unknown.
    #[error("invalid receiver type: {value}")]
    InvalidReceiverTypeString { value: String },

    /// Receiver type combination is unsupported.
    #[error("receiver types do not produce a valid unified address")]
    InvalidReceiverCombination,

    /// Missing UFVK metadata for a given fingerprint.
    #[error("missing unified full viewing key for fingerprint {fingerprint}")]
    MissingUfvk { fingerprint: String },

    /// Orchard incoming viewing key is invalid.
    #[error("invalid Orchard incoming viewing key")]
    InvalidOrchardIncomingViewingKey,

    /// Unexpected value encountered while parsing metadata.
    #[error("unexpected {kind} value: 0x{value:08x}")]
    UnexpectedValue { kind: &'static str, value: u32 },

    /// Key/value records were mismatched in the wallet dump.
    #[error("mismatched {kind} records")]
    MismatchedRecords { kind: &'static str },

    /// Public/private keypair mismatch.
    #[error("pubkey and privkey hash do not match")]
    InvalidKeypair,

    /// Unsupported TEX address encountered.
    #[error("unsupported TEX address encountered: {address}")]
    UnsupportedAddressType { address: String },

    /// Address identifier string is malformed.
    #[error("invalid address identifier: {input}")]
    InvalidAddressIdFormat { input: String },

    /// Bit pattern is invalid for the given type.
    #[error("invalid bit pattern for {kind}")]
    InvalidBitPattern { kind: &'static str },

    /// CompactSize encoding used an invalid prefix/value combination.
    #[error("invalid CompactSize prefix {prefix:#04x} with value {value}")]
    InvalidCompactSize { prefix: u8, value: u64 },
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Helper for constructing a boxed-context variant without repeating
    /// boilerplate.
    pub fn with_context(
        source: impl StdError + Send + Sync + 'static,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Error::Context { message: message.into(), source: Box::new(source) }
    }
}

pub trait ResultExt<T> {
    fn context(self, msg: impl Into<Cow<'static, str>>) -> Result<T>;
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: StdError + Send + Sync + 'static,
{
    fn context(self, msg: impl Into<Cow<'static, str>>) -> Result<T> {
        self.map_err(|err| Error::with_context(err, msg.into()))
    }

    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|err| Error::with_context(err, f()))
    }
}

pub trait OptionExt<T> {
    fn context<E>(self, err: E) -> Result<T>
    where
        E: FnOnce() -> Error;
}

impl<T> OptionExt<T> for Option<T> {
    fn context<E>(self, err: E) -> Result<T>
    where
        E: FnOnce() -> Error,
    {
        self.ok_or_else(err)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DumpInconsistency {
    UnmatchedKeyValue,
    NonUniqueKeys,
}

impl fmt::Display for DumpInconsistency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DumpInconsistency::UnmatchedKeyValue => {
                write!(f, "found key without corresponding value")
            }
            DumpInconsistency::NonUniqueKeys => {
                write!(f, "non-unique keys detected")
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExpectedLengths {
    Single(usize),
    Multiple(&'static [usize]),
}

impl fmt::Display for ExpectedLengths {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpectedLengths::Single(value) => write!(f, "{value}"),
            ExpectedLengths::Multiple(values) => {
                if values.len() == 1 {
                    write!(f, "{}", values[0])
                } else {
                    let mut first = true;
                    for v in *values {
                        if !first {
                            write!(f, ", ")?;
                        }
                        write!(f, "{v}")?;
                        first = false;
                    }
                    Ok(())
                }
            }
        }
    }
}
