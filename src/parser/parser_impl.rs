//! Core binary parsing infrastructure for the ZeWIF codebase.
//!
//! This module defines the fundamental traits and structures for binary deserialization,
//! providing a consistent interface for parsing Zcash wallet and blockchain data from
//! raw bytes. It includes both the low-level `Parser` for byte manipulation and the
//! higher-level `Parse` and `ParseWithParam` traits for structured type parsing.

use anyhow::{Result, bail};

use zewif::Data;

/// A trait for types that can be parsed from a binary data stream.
///
/// The `Parse` trait defines a standard interface for deserializing Zcash data types
/// from binary data. Types implementing this trait can be used with the `parse!` macro
/// and the Parser infrastructure for consistent binary deserialization.
///
/// # Zcash Concept Relation
/// Zcash data structures are frequently serialized and deserialized in binary format
/// for blockchain storage, network transmission, and wallet persistence. The `Parse`
/// trait provides a uniform way to handle these operations across all types in the
/// ZeWIF codebase.
///
/// # Examples
/// ```no_run
/// # use anyhow::Result;
/// # use zewif_zcashd::parser::{Parse, Parser};
/// #
/// // Implementing Parse for a custom type
/// struct MyType {
///     value: u32,
///     name: String,
/// }
///
/// impl Parse for MyType {
///     fn parse(p: &mut Parser) -> Result<Self> {
///         let value = u32::parse(p)?;
///         let name = String::parse(p)?;
///         Ok(Self { value, name })
///     }
/// }
/// ```
pub trait Parse {
    /// Parses an instance of this type from a parser's binary data stream.
    ///
    /// This is the primary method that must be implemented to satisfy the `Parse` trait.
    /// It should read exactly the number of bytes needed to reconstruct the type.
    fn parse(p: &mut Parser) -> Result<Self>
    where
        Self: Sized;

    /// Parses an instance of this type from a complete byte buffer.
    ///
    /// This convenience method creates a `Parser` from the provided buffer,
    /// parses the type, and ensures that the entire buffer was consumed.
    ///
    /// # Arguments
    /// * `buf` - The buffer containing the binary data to parse
    /// * `trace` - Whether to enable debug tracing during parsing
    ///
    /// # Returns
    /// The parsed instance if successful
    ///
    /// # Errors
    /// Returns an error if parsing fails or if there are unconsumed bytes in the buffer
    fn parse_buf(buf: &dyn AsRef<[u8]>, trace: bool) -> Result<Self>
    where
        Self: Sized,
    {
        let mut p = Parser::new(&buf);
        p.set_trace(trace);
        let result = Self::parse(&mut p)?;
        p.check_finished()?;
        Ok(result)
    }
}

/// A trait for types that require additional parameters during parsing.
///
/// The `ParseWithParam` trait extends the `Parse` concept to accommodate types that
/// need contextual information beyond what's available in the binary data stream itself.
/// This is particularly useful for polymorphic types or parsing decisions that depend
/// on previously parsed data.
///
/// # Zcash Concept Relation
/// Some Zcash data structures require contextual information for proper parsing.
/// For example, the `SproutProof` type needs to know whether to parse a PHGR proof
/// or a Groth16 proof based on flags set elsewhere in the transaction.
///
/// # Examples
/// ```no_run
/// # use zewif_zcashd::parser::prelude::*;
/// # use anyhow::Result;
/// #
/// // A type that needs a parameter during parsing
/// enum ProofType {
///     TypeA(u32),
///     TypeB(u64),
/// }
///
/// impl ParseWithParam<bool> for ProofType {
///     fn parse(p: &mut Parser, use_type_a: bool) -> Result<Self> {
///         if use_type_a {
///             Ok(Self::TypeA(u32::parse(p)?))
///         } else {
///             Ok(Self::TypeB(u64::parse(p)?))
///         }
///     }
/// }
/// ```
pub trait ParseWithParam<P> {
    /// Parses an instance of this type from a parser's binary data stream,
    /// using the provided parameter for context.
    ///
    /// This is the primary method that must be implemented to satisfy the `ParseWithParam` trait.
    /// It should use the parameter to make parsing decisions as needed.
    fn parse(p: &mut Parser, param: P) -> Result<Self>
    where
        Self: Sized;

    /// Parses an instance of this type from a complete byte buffer,
    /// using the provided parameter for context.
    ///
    /// This convenience method creates a `Parser` from the provided buffer,
    /// parses the type with the parameter, and ensures that the entire buffer was consumed.
    ///
    /// # Arguments
    /// * `buf` - The buffer containing the binary data to parse
    /// * `param` - The parameter to use during parsing
    /// * `trace` - Whether to enable debug tracing during parsing
    ///
    /// # Returns
    /// The parsed instance if successful
    ///
    /// # Errors
    /// Returns an error if parsing fails or if there are unconsumed bytes in the buffer
    #[allow(dead_code)]
    fn parse_buf(buf: &dyn AsRef<[u8]>, param: P, trace: bool) -> Result<Self>
    where
        Self: Sized,
    {
        let mut p = Parser::new(&buf);
        p.set_trace(trace);
        let result = Self::parse(&mut p, param)?;
        p.check_finished()?;
        Ok(result)
    }
}

/// A binary data stream parser for Zcash wallet and blockchain data.
///
/// The `Parser` struct provides low-level byte manipulation capabilities for
/// deserializing Zcash binary data. It maintains a buffer of bytes and tracks
/// the current parsing position, offering methods to read, peek, and navigate
/// through the data stream.
///
/// # Zcash Concept Relation
/// Zcash wallet and blockchain data is typically serialized in binary formats,
/// often with complex nested structures. The `Parser` provides the foundation
/// for reading these formats, supporting both transparent Bitcoin-compatible
/// structures and Zcash-specific shielded constructs.
///
/// # Examples
/// ```no_run
/// # use zewif_zcashd::parser::prelude::*;
/// # use anyhow::Result;
/// #
/// # fn example() -> Result<()> {
/// // Create a parser from raw bytes
/// let data = vec![0x04, 0x01, 0x02, 0x03, 0x04];
/// let mut parser = Parser::new(&data);
///
/// // Read the first byte (length)
/// let length_bytes = parser.next(1)?;
/// let length = length_bytes[0];
/// assert_eq!(length, 4);
///
/// // Read the remaining bytes
/// let payload = parser.next(4)?;
/// assert_eq!(payload, &[1, 2, 3, 4]);
///
/// // Parser is now at the end of the buffer
/// assert_eq!(parser.remaining(), 0);
/// # Ok(())
/// # }
/// ```
pub struct Parser<'a> {
    /// The byte buffer being parsed
    pub buffer: &'a [u8],

    /// Current position within the buffer
    pub offset: usize,

    /// Whether to print debug information during parsing
    pub trace: bool,
}

impl std::fmt::Debug for Parser<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Parser")
            .field("offset", &self.offset)
            .field("len", &self.len())
            .field("remaining", &self.remaining())
            .finish()
    }
}

impl<'a> Parser<'a> {
    pub fn new(buffer: &'a dyn AsRef<[u8]>) -> Self {
        Self {
            buffer: buffer.as_ref(),
            offset: 0,
            trace: false,
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn remaining(&self) -> usize {
        self.len() - self.offset
    }

    pub fn check_finished(&self) -> Result<()> {
        if self.offset < self.buffer.len() {
            bail!("Buffer has {} bytes left", self.remaining());
        }
        Ok(())
    }

    pub fn next(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.offset + n > self.buffer.len() {
            bail!(
                "Buffer underflow at offset {}, needed {} bytes, only {} remaining",
                self.offset,
                n,
                self.remaining()
            );
        }
        let bytes = &self.buffer[self.offset..self.offset + n];
        self.offset += n;
        if self.trace {
            println!(
                "\t🟢 next({}): {:?} remaining: {} peek: {:?}",
                n,
                hex::encode(bytes),
                self.remaining(),
                hex::encode(self.peek(100))
            );
        }
        Ok(bytes)
    }

    pub fn peek(&self, n: usize) -> &'a [u8] {
        let available = std::cmp::min(n, self.remaining());
        &self.buffer[self.offset..self.offset + available]
    }

    pub fn rest(&mut self) -> Data {
        let bytes = self.next(self.remaining()).unwrap();
        Data::from_slice(bytes)
    }

    pub fn peek_rest(&self) -> Data {
        Data::from_slice(&self.buffer[self.offset..])
    }

    pub fn set_trace(&mut self, trace: bool) {
        self.trace = trace;
    }

    pub fn trace(&self, msg: &str) {
        if self.trace {
            println!("🔵 {}: {:?}", msg, self.peek_rest());
        }
    }
}

impl std::io::Read for &mut Parser<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let parser = &mut **self;
        let n = std::cmp::min(buf.len(), parser.remaining());
        buf[..n].copy_from_slice(&parser.buffer[parser.offset..parser.offset + n]);
        parser.offset += n;
        Ok(n)
    }
}
