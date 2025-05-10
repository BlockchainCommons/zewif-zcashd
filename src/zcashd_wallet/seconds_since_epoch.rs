use anyhow::Result;
use chrono::{SecondsFormat, TimeZone, Utc};

use crate::{parse, parser::prelude::*};

/// A timestamp measured as seconds since the Unix epoch (1970-01-01T00:00:00Z).
///
/// `SecondsSinceEpoch` represents a point in time as a count of seconds since
/// January 1, 1970, 00:00:00 UTC (the Unix epoch). This type is used for
/// transaction timestamps and other time-related data in the ZeWIF format.
///
/// # Zcash Concept Relation
/// In Zcash, timestamps are used to:
/// - Record when transactions were created or mined
/// - Track block times in the blockchain
/// - Provide time-based information for wallet synchronization
///
/// Like Bitcoin, Zcash uses Unix timestamp format for all time-related data.
///
/// # Data Preservation
/// This type preserves the exact timestamp values from wallet data files,
/// ensuring that transaction history maintains its correct chronological order
/// during wallet migration.
///
/// # Examples
/// ```
/// # use zewif_zcashd::zcashd_wallet::SecondsSinceEpoch;
/// // Create a timestamp for January 1, 2023
/// let jan_1_2023 = SecondsSinceEpoch::from(1672531200u64);
///
/// // Check if timestamp is zero (for uninitialized or default values)
/// assert!(!jan_1_2023.is_zero());
///
/// // Display formats the timestamp as an ISO-8601 date-time: "2023-01-01T00:00:00Z"
/// println!("Timestamp: {}", jan_1_2023);
/// ```
///
/// The internal value is stored as a 64-bit unsigned integer, allowing for timestamps
/// well beyond the year 2038 (unlike 32-bit Unix timestamps which have the Y2038 problem).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SecondsSinceEpoch(u64);

impl SecondsSinceEpoch {
    /// Returns `true` if this timestamp is zero.
    ///
    /// A zero timestamp is often used as a default or null value in blockchain data
    /// structures, similar to how Unix systems sometimes use a zero timestamp to indicate
    /// uninitialized or unknown time values.
    ///
    /// # Examples
    /// ```
    /// # use zewif_zcashd::zcashd_wallet::SecondsSinceEpoch;
    /// let zero_time = SecondsSinceEpoch::from(0u64);
    /// assert!(zero_time.is_zero());
    ///
    /// let non_zero_time = SecondsSinceEpoch::from(1672531200u64);
    /// assert!(!non_zero_time.is_zero());
    /// ```
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

/// Creates a timestamp from a u64 seconds value
impl From<u64> for SecondsSinceEpoch {
    fn from(seconds: u64) -> Self {
        Self(seconds)
    }
}

/// Extracts the raw u64 seconds value from a timestamp
impl From<SecondsSinceEpoch> for u64 {
    fn from(seconds: SecondsSinceEpoch) -> Self {
        seconds.0
    }
}

/// Creates a timestamp from a u32 seconds value
///
/// This is useful for compatibility with 32-bit timestamp formats used in
/// some parts of the Bitcoin/Zcash protocols.
impl From<u32> for SecondsSinceEpoch {
    fn from(seconds: u32) -> Self {
        Self(seconds as u64)
    }
}

/// Parses a SecondsSinceEpoch from a binary data stream
impl Parse for SecondsSinceEpoch {
    fn parse(p: &mut Parser) -> Result<Self> {
        let seconds = parse!(p, u64, "seconds")?;
        Ok(SecondsSinceEpoch(seconds))
    }
}

/// Formats the timestamp as an ISO-8601 date-time string (e.g., "2023-01-01T00:00:00Z")
impl std::fmt::Debug for SecondsSinceEpoch {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let dt = Utc
            .timestamp_opt(self.0 as i64, 0)
            .single()
            .unwrap()
            .to_rfc3339_opts(SecondsFormat::Secs, true);
        write!(f, "{}", dt)
    }
}

/// Displays the timestamp in the same ISO-8601 format as Debug
impl std::fmt::Display for SecondsSinceEpoch {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
