use anyhow::{Result, bail};

use crate::{parse, parser::prelude::*};

/// Parses a Bitcoin-style variable-length integer (compact size) from a binary data stream.
///
/// This function implements the Bitcoin/Zcash compact size encoding, where:
/// - Values 0-252 are encoded as a single byte
/// - Values 253-65535 are encoded as 0xfd followed by 2 bytes (little-endian)
/// - Values 65536-4294967295 are encoded as 0xfe followed by 4 bytes (little-endian)
/// - Values 4294967296+ are encoded as 0xff followed by 8 bytes (little-endian)
///
/// This encoding scheme optimizes space usage for small integers while allowing for very
/// large values when needed.
///
/// # Arguments
/// * `p` - A mutable reference to a Parser to read bytes from
///
/// # Returns
/// The parsed size value as a usize, or an error if parsing fails.
///
/// # Errors
/// Returns an error if:
/// - There are insufficient bytes in the parser input
/// - The encoding doesn't follow the compact size rules (e.g., using 0xfd for a value < 253)
///
/// # Examples
/// ```no_run
/// # use zewif_zcashd::{zcashd_wallet::parse_compact_size, parser::prelude::*};
/// # use anyhow::Result;
/// #
/// # fn example() -> Result<()> {
/// // Create a buffer with compact size data
/// let data = vec![0u8; 10]; // Example data
///
/// // Create a parser
/// let mut parser = Parser::new(&data);
///
/// // Parse a compact size from the binary data
/// let size = parse_compact_size(&mut parser)?;
/// println!("Parsed compact size: {}", size);
/// # Ok(())
/// # }
/// # fn main() {}
/// ```
pub fn parse_compact_size(p: &mut Parser) -> Result<usize> {
    match parse!(p, u8, "compact size")? {
        0xfd => {
            let n = parse!(p, u16, "compact size")?;
            if n < 253 {
                bail!("Compact size with 0xfd prefix must be >= 253, got {}", n);
            }
            Ok(n as usize)
        }
        0xfe => {
            let n = parse!(p, u32, "compact size")?;
            if n < 0x10000 {
                bail!(
                    "Compact size with 0xfe prefix must be >= 0x10000, got {}",
                    n
                );
            }
            Ok(n as usize)
        }
        0xff => {
            let n = parse!(p, u64, "compact size")?;
            if n < 0x100000000 {
                bail!(
                    "Compact size with 0xff prefix must be >= 0x100000000, got {}",
                    n
                );
            }
            Ok(n as usize)
        }
        size => Ok(size as usize),
    }
}

/// A Bitcoin/Zcash-style variable-length integer used for size encoding in binary formats.
///
/// `CompactSize` is a wrapper around a `usize` that represents a value encoded in the
/// Bitcoin/Zcash variable-length integer format. This format is used throughout the
/// Zcash protocol to encode lengths of arrays, strings, and other variable-length data.
///
/// # Zcash Concept Relation
/// The compact size encoding is used extensively in Zcash's binary formats, including:
/// - Transaction data serialization
/// - Block serialization
/// - Network message formats
/// - Wallet data storage
///
/// It optimizes space usage by using fewer bytes for smaller values while still
/// supporting very large values when needed.
///
/// # Data Preservation
/// `CompactSize` ensures that variable-length data is correctly interpreted during
/// wallet migration, preserving the exact encoding used in the original wallet format.
///
/// # Technical Implementation
/// Internally, `CompactSize` wraps a `usize` and implements `std::ops::Deref` to allow
/// using it where a `usize` is expected. It also provides implementations of `Parse`
/// for binary data parsing.
///
/// # Examples
/// ```no_run
/// # use zewif_zcashd::{parse, parser::prelude::*, zcashd_wallet::CompactSize};
/// # use anyhow::Result;
/// #
/// # fn example() -> Result<()> {
/// // Create a buffer with binary data
/// let data = vec![0u8; 10]; // Example data
///
/// // Create a parser
/// let mut p = Parser::new(&data);
///
/// // Parse a compact size from the binary stream
/// let size: CompactSize = parse!(&mut p, CompactSize, "vector length")?;
///
/// // Use the value as a regular usize
/// let vec_length: usize = *size;
/// println!("Vector length: {}", size);
/// # Ok(())
/// # }
/// # fn main() {}
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompactSize(usize);

impl std::fmt::Display for CompactSize {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Parse for CompactSize {
    fn parse(p: &mut Parser) -> Result<Self> {
        parse_compact_size(p).map(CompactSize)
    }
}

impl std::ops::Deref for CompactSize {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
