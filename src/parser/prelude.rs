//! # Parser Prelude
//!
//! This module provides a convenient single import for all essential parsing
//! components. It's designed as a "batteries included" import to simplify
//! parser usage throughout the codebase.
//!
//! ## Included Components
//!
//! - **Core Parser Types**: The `Parser` struct and `Parse` trait from `parser_impl`
//! - **Parseable Type Implementations**: Standard parsers from `parseable_types`
//!
//! ## Usage
//!
//! Import this module when implementing parsing for new types:
//!
//! ```ignore
//! use zewif_zcashd::parser::prelude::*;
//! use anyhow::Result;
//!
//! struct MyType {}
//!
//! impl Parse for MyType {
//!     fn parse(parser: &mut Parser) -> Result<Self> {
//!         // Implementation using standard parsers from the prelude
//!         Ok(MyType {})
//!     }
//! }
//! ```
//!
//! This prelude pattern follows Rust's convention (like the standard library's
//! `std::prelude`) of providing the most commonly needed imports in a single module.

#[doc(hidden)]
pub use super::parser_impl::*;
#[doc(hidden)]
pub use crate::zcashd_wallet::*;
