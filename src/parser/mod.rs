//! # Binary Parsing Infrastructure
//!
//! This module provides a comprehensive toolkit for parsing binary wallet data into
//! strongly-typed Rust structures. It's designed to handle the complexities of Zcash's
//! binary encoding formats with robust error handling and context-aware parsing.
//!
//! ## Key Components
//!
//! - **Parser**: The core parsing engine that tracks position and context in a binary stream
//! - **Parse Trait**: A standardized interface for types that can be parsed from binary data
//! - **parse! Macro**: A context-aware macro that simplifies parsing with improved error messages
//! - **Standard Implementations**: Built-in parsers for common types (numbers, vectors, etc.)
//!
//! ## Design Philosophy
//!
//! The parsing system is designed with several key principles:
//!
//! 1. **Type Safety**: Each parsed type has a dedicated implementation with appropriate validation
//! 2. **Contextual Errors**: Error messages include both the type being parsed and the context
//! 3. **Composition**: Complex types are built by combining parsers for simpler types
//! 4. **Extensibility**: New types can easily implement the `Parse` trait
//!
//! ## Usage
//!
//! The primary entry point is the `parse!` macro, which provides a clean syntax for parsing.
//! The `parser::prelude` module includes all necessary components for parsing, making it easy
//! to get started. Note that they *do* require separate imports, as shown below.
//!
//! ```no_run
//! use zewif::{TxId, Transaction};
//! use zewif_zcashd::{parse, parser::prelude::*};
//! use anyhow::Result;
//!
//! # fn example(data: &[u8]) -> Result<()> {
//! let mut parser = Parser::new(&data);
//!
//! // Parse a u32 with context
//! let block_height = parse!(&mut parser, u32, "block height")?;
//!
//! // Parse a complex type with context
//! let txid = parse!(&mut parser, TxId, "transaction ID")?;
//! let transaction = Transaction::new(txid);
//! # Ok(())
//! # }
//! ```
//!
//! ## Module Structure
//!
//! - [`parse_macro`]: Defines the `parse!` macro for context-aware parsing
//! - `parser_impl`: Core parser implementation and the `Parse` trait definition
//! - `parseable_types`: Standard implementations of the `Parse` trait for common types
//! - [`prelude`]: Common imports for convenient parser usage

#![allow(unused_imports)]

use crate::mod_use;

pub(crate) mod parse_macro;
pub mod prelude;

mod_use!(parser_impl);
