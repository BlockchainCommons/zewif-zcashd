/// A macro for parsing binary data with context-aware error messages.
///
/// The `parse!` macro provides a convenient way to parse binary data from a parser
/// or buffer while adding helpful context information to any errors that might occur.
/// It works with types that implement the `Parse` or `ParseWithParam` traits, or with
/// specialized parsing functions.
///
/// # Usage Patterns
///
/// ## Basic Type Parsing
/// Parse a type that implements the `Parse` trait:
/// ```no_run
/// # use zewif_zcashd::{parser::prelude::*, parse, zcashd_wallet::CompactSize};
/// # use zewif_zcashd::Result;
/// # fn example(parser: &mut Parser) -> Result<()> {
/// let size = parse!(parser, CompactSize, "transaction size")?;
/// # Ok(())
/// # }
/// ```
///
/// ## Parsing Data with a Fixed Length
/// Parse a fixed-length byte array or Data object:
/// ```no_run
/// # use zewif::Data;
/// # use zewif_zcashd::{parser::prelude::*, parse};
/// # use zewif_zcashd::Result;
/// # fn example(parser: &mut Parser) -> Result<()> {
/// // Parse 32 bytes (e.g. for a hash)
/// let bytes = parse!(parser, bytes = 32, "transaction hash")?;
/// // Or parse into a Data object
/// let data = parse!(parser, data = 32, "signature data")?;
/// # Ok(())
/// # }
/// ```
///
/// ## Parsing with Parameters
/// Parse a type that implements `ParseWithParam` and needs additional parameters:
/// ```no_run
/// # use zewif_zcashd::{parser::prelude::*, parse};
/// # use zewif_zcashd::Result;
/// #
/// # // Define a dummy type that implements ParseWithParam for the example
/// # struct SomeType;
/// # impl ParseWithParam<u32> for SomeType {
/// #     fn parse(_parser: &mut Parser, _param: u32) -> Result<Self> { Ok(SomeType) }
/// #     fn parse_buf(_buf: &dyn AsRef<[u8]>, _param: u32, _trace: bool) -> Result<Self> { Ok(SomeType) }
/// # }
/// #
/// # fn example(parser: &mut Parser, param: u32) -> Result<()> {
/// let value = parse!(parser, SomeType, param = param, "parameterized type")?;
/// # Ok(())
/// # }
/// ```
///
/// # Error Handling
/// The macro automatically adds context to errors, making debugging easier by
/// describing what was being parsed when an error occurred.
///
/// # Relation to ZCash Data Formats
/// This macro is particularly useful when parsing ZCash wallet and transaction data,
/// which often involves nested structures with complex parsing rules. The context
/// provided helps identify which part of a structure failed to parse.
#[macro_export]
macro_rules! parse {
    (buf = $buf:expr, $type:ty, $context:expr) => {{
        <$type as $crate::parser::Parse>::parse_buf($buf, false).map_err(
            |err| {
                $crate::Error::with_context(
                    err,
                    format!("Parsing {}", $context),
                )
            },
        )
    }};
    (buf = $buf:expr, $type:ty, param = $param:expr, $context:expr) => {{
        <$type as $crate::parser::ParseWithParam<_>>::parse_buf(
            $buf, $param, false,
        )
        .map_err(|err| {
            $crate::Error::with_context(err, format!("Parsing {}", $context))
        })
    }};
    (buf = $buf:expr, $type:ty, $context:expr, $trace:expr) => {{
        <$type as $crate::parser::Parse>::parse_buf($buf, $trace).map_err(
            |err| {
                $crate::Error::with_context(
                    err,
                    format!("Parsing {}", $context),
                )
            },
        )
    }};
    (buf = $buf:expr, $type:ty, param = $param:expr, $context:expr, $trace:expr) => {{
        <$type as $crate::parser::ParseWithParam<_>>::parse_buf(
            $buf, $param, $trace,
        )
        .map_err(|err| {
            $crate::Error::with_context(err, format!("Parsing {}", $context))
        })
    }};
    ($parser:expr, $type:ty, $context:expr) => {{
        <$type as $crate::parser::Parse>::parse($parser).map_err(|err| {
            $crate::Error::with_context(err, format!("Parsing {}", $context))
        })
    }};
    ($parser:expr, $type:ty, param = $param:expr, $context:expr) => {{
        <$type as $crate::parser::ParseWithParam<_>>::parse($parser, $param)
            .map_err(|err| {
                $crate::Error::with_context(
                    err,
                    format!("Parsing {}", $context),
                )
            })
    }};
    ($parser:expr, bytes = $length:expr, $context:expr) => {{
        $crate::parser::Parser::next($parser, $length).map_err(|err| {
            $crate::Error::with_context(err, format!("Parsing {}", $context))
        })
    }};
    ($parser:expr, data = $length:expr, $context:expr) => {{
        $crate::parser::Parser::next($parser, $length)
            .map(zewif::Data::from_slice)
            .map_err(|err| {
                $crate::Error::with_context(
                    err,
                    format!("Parsing {}", $context),
                )
            })
    }};
    ($parser:expr, $context:expr) => {{
        $crate::parser::Parse::parse($parser).map_err(|err| {
            $crate::Error::with_context(err, format!("Parsing {}", $context))
        })
    }};
    ($parser:expr, param = $param:expr, $context:expr) => {{
        $crate::parser::ParseWithParam::parse($parser, $param).map_err(|err| {
            $crate::Error::with_context(err, format!("Parsing {}", $context))
        })
    }};
}

#[macro_export]
macro_rules! blob_parse {
    ($name:ident, $size:expr) => {
        impl $crate::parser::Parse for $name {
            /// Parses this type from a binary data stream.
            ///
            /// This implementation allows the type to be used with the `parse!` macro.
            fn parse(
                parser: &mut $crate::parser::Parser,
            ) -> $crate::Result<Self> {
                let bytes =
                    $crate::parse!(parser, bytes = $size, stringify!($name))?;
                Ok(Self::from_slice(bytes)?)
            }
        }
    };
}

#[macro_export]
macro_rules! data_parse {
    ($name:ident) => {
        impl $crate::parser::Parse for $name {
            /// Parses this type from a binary data stream.
            ///
            /// This implementation allows the type to be used with the `parse!` macro.
            /// The data is parsed as a length-prefixed byte array using a `CompactSize`
            /// value to indicate the length.
            fn parse(
                parser: &mut $crate::parser::Parser,
            ) -> $crate::Result<Self> {
                let data = $crate::parse!(parser, Data, stringify!($name))?;
                Ok(Self(data))
            }
        }
    };
}

#[macro_export]
macro_rules! string_parse {
    ($name:ident) => {
        impl $crate::parser::Parse for $name {
            /// Parses this type from a binary data stream.
            ///
            /// This implementation allows the type to be used with the `parse!` macro.
            /// The string is assumed to be encoded in the binary format as a length-prefixed
            /// UTF-8 string.
            fn parse(p: &mut $crate::parser::Parser) -> $crate::Result<Self> {
                Ok(Self($crate::parse!(p, String, "string")?))
            }
        }
    };
}
