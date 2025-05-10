use anyhow::{Result, anyhow};
use bitflags::bitflags;
use std::{
    convert::Infallible,
    fmt::{self, Display, Formatter},
    str::FromStr,
};
use zcash_address::{ConversionError, TryFromAddress, ZcashAddress};
use zcash_protocol::consensus::NetworkType;

use zewif::ProtocolAddress;

use crate::zcashd_wallet::{ReceiverType, UfvkFingerprint, UnifiedAddressMetadata};

bitflags! {
    /// A set of flags describing the type(s) of outputs that a Zcash address can receive.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ReceiverFlags: i64 {
        /// The address did not contain any recognized receiver types.
        const UNKNOWN = 0b00000000;
        /// The associated address can receive transparent p2pkh outputs.
        const P2PKH = 0b00000001;
        /// The associated address can receive transparent p2sh outputs.
        const P2SH = 0b00000010;
        /// The associated address can receive Sapling outputs.
        const SAPLING = 0b00000100;
        /// The associated address can receive Orchard outputs.
        const ORCHARD = 0b00001000;
    }
}

enum AddressType {
    Sprout,
    Sapling,
    Unified,
    P2pkh,
    P2sh,
    Tex,
}

impl TryFromAddress for AddressType {
    type Error = Infallible;

    fn try_from_sprout(_: NetworkType, _: [u8; 64]) -> Result<Self, ConversionError<Self::Error>> {
        Ok(AddressType::Sprout)
    }

    fn try_from_sapling(_: NetworkType, _: [u8; 43]) -> Result<Self, ConversionError<Self::Error>> {
        Ok(AddressType::Sapling)
    }

    fn try_from_unified(
        _: NetworkType,
        _: zcash_address::unified::Address,
    ) -> Result<Self, ConversionError<Self::Error>> {
        Ok(AddressType::Unified)
    }

    fn try_from_transparent_p2pkh(
        _: NetworkType,
        _: [u8; 20],
    ) -> Result<Self, ConversionError<Self::Error>> {
        Ok(AddressType::P2pkh)
    }

    fn try_from_transparent_p2sh(
        _: NetworkType,
        _: [u8; 20],
    ) -> Result<Self, ConversionError<Self::Error>> {
        Ok(AddressType::P2sh)
    }

    fn try_from_tex(_: NetworkType, _: [u8; 20]) -> Result<Self, ConversionError<Self::Error>> {
        Ok(AddressType::Tex)
    }
}

/// A universal identifier for addresses across different Zcash protocols.
///
/// `AddressId` provides a common interface for working with addresses from all Zcash
/// protocols: transparent, Sapling, and unified addresses. This type serves
/// as a key abstraction in wallet data migration, allowing addresses to be tracked
/// consistently regardless of their underlying protocol type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AddressId {
    /// Transparent address (P2PKH or P2SH)
    Transparent(String),
    /// Sapling address
    Sprout(String),
    /// Sapling address
    Sapling(String),
    /// Unified address
    Unified(String),
    /// Derivation metadata for a unified address.
    DerivationMeta {
        ufvk_id: UfvkFingerprint,
        diversifier_index: [u8; 11],
        receiver_types: ReceiverFlags,
    },
}

impl AddressId {
    /// Creates a new `AddressId` from a `ProtocolAddress`.
    ///
    /// This converts a protocol-specific address into a universal identifier,
    /// automatically determining the correct address type based on the input.
    pub fn from_protocol_address(address: &ProtocolAddress) -> Self {
        match address {
            ProtocolAddress::Transparent(addr) => {
                AddressId::Transparent(addr.address().to_string())
            }
            ProtocolAddress::Sapling(addr) => AddressId::Sapling(addr.address().to_string()),
            ProtocolAddress::Unified(addr) => AddressId::Unified(addr.address().to_string()),
        }
    }

    pub fn from_unified_address_metadata(meta: &UnifiedAddressMetadata) -> Self {
        let mut receiver_flags = ReceiverFlags::empty();
        for t in &meta.receiver_types {
            receiver_flags |= match t {
                ReceiverType::P2PKH => ReceiverFlags::P2PKH,
                ReceiverType::P2SH => ReceiverFlags::P2SH,
                ReceiverType::Sapling => ReceiverFlags::SAPLING,
                ReceiverType::Orchard => ReceiverFlags::ORCHARD,
            }
        }
        AddressId::DerivationMeta {
            ufvk_id: meta.key_id,
            diversifier_index: meta.diversifier_index.clone().into(),
            receiver_types: receiver_flags,
        }
    }

    /// Creates a new `AddressId` from a string representation of an address and network information.
    ///
    /// This method determines the address type based on the address prefix:
    /// - 't' for transparent addresses
    /// - 'zs' for Sapling addresses
    /// - 'u' for unified addresses
    pub fn from_address_string(addr_str: &str) -> Result<Self> {
        let decoded = ZcashAddress::try_from_encoded(addr_str)?;
        match decoded.convert::<AddressType>()? {
            AddressType::Sprout => Ok(Self::Sapling(addr_str.to_string())),
            AddressType::Sapling => Ok(Self::Sapling(addr_str.to_string())),
            AddressType::P2pkh | AddressType::P2sh => Ok(Self::Transparent(addr_str.to_string())),
            AddressType::Unified => Ok(Self::Unified(addr_str.to_string())),
            AddressType::Tex => Err(anyhow!(
                "TEX addresses do not occur in zcashd address data."
            )),
        }
    }

    /// Get the address string if this is a directly addressable address
    pub fn address_string(&self) -> Option<&str> {
        match self {
            AddressId::Transparent(addr) => Some(addr),
            AddressId::Sapling(addr) => Some(addr),
            AddressId::Unified(addr) => Some(addr),
            AddressId::Sprout(addr) => Some(addr),
            AddressId::DerivationMeta { .. } => None,
        }
    }

    /// Returns the address protocol type as a string
    pub fn protocol_type(&self) -> &'static str {
        match self {
            AddressId::Transparent(_) => "transparent",
            AddressId::Sapling(_) => "sapling",
            AddressId::Unified(_) => "unified",
            AddressId::DerivationMeta { .. } => "unified",
            AddressId::Sprout(_) => "sprout",
        }
    }
}

impl Display for AddressId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AddressId::Transparent(addr) => write!(f, "t:{}", addr),
            AddressId::Sapling(addr) => write!(f, "zs:{}", addr),
            AddressId::Unified(addr) => write!(f, "u:{}", addr),
            AddressId::Sprout(addr) => write!(f, "sprout:{}", addr),
            AddressId::DerivationMeta { .. } => write!(f, "u:<not_rendered>"),
        }
    }
}

impl FromStr for AddressId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(addr) = s.strip_prefix("t:") {
            Ok(Self::Transparent(addr.to_string()))
        } else if let Some(addr) = s.strip_prefix("zs:") {
            Ok(Self::Sapling(addr.to_string()))
        } else if let Some(addr) = s.strip_prefix("u:") {
            Ok(Self::Unified(addr.to_string()))
        //        } else if let Some(id) = s.strip_prefix("ua:") {
        //            // Parse the u256 value
        //            let mut id_bytes =
        //                hex::decode(id).context("Invalid hex encoding for unified account ID")?;
        //            id_bytes.reverse();
        //            let account_id = UfvkFingerprint::from_bytes(&id_bytes)?;
        //            Ok(Self::UnifiedAccountAddress(account_id))
        } else {
            Err(anyhow::anyhow!("Invalid AddressId format: {}", s))
        }
    }
}

/// A registry that tracks address-to-account mappings during wallet migration.
///
/// `AddressRegistry` maintains a bidirectional mapping between addresses and accounts,
/// allowing wallet migration tools to properly associate addresses with their respective
/// accounts. This is particularly important for wallets with multiple accounts or
/// unified accounts with multiple address types.
#[derive(Debug, Default)]
pub(crate) struct AddressRegistry {
    // Maps from AddressId to account identifier (u256)
    address_to_account: std::collections::HashMap<AddressId, UfvkFingerprint>,
}

impl AddressRegistry {
    /// Create a new, empty address registry
    pub(crate) fn new() -> Self {
        Self {
            address_to_account: std::collections::HashMap::new(),
        }
    }

    /// Register an address with an account
    pub(crate) fn register(&mut self, address_id: AddressId, account_id: UfvkFingerprint) {
        self.address_to_account.insert(address_id, account_id);
    }

    /// Find the account ID for a given address
    pub(crate) fn find_account(&self, address_id: &AddressId) -> Option<&UfvkFingerprint> {
        self.address_to_account.get(address_id)
    }

    /// Find all addresses belonging to a specific account
    pub(crate) fn find_addresses_for_account(
        &self,
        account_id: &UfvkFingerprint,
    ) -> Vec<&AddressId> {
        self.address_to_account
            .iter()
            .filter_map(|(addr_id, acct_id)| {
                if acct_id == account_id {
                    Some(addr_id)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns the number of registered addresses
    pub(crate) fn address_count(&self) -> usize {
        self.address_to_account.len()
    }

    /// Returns the number of unique accounts referenced
    pub(crate) fn account_count(&self) -> usize {
        self.address_to_account
            .values()
            .collect::<std::collections::HashSet<_>>()
            .len()
    }
}

#[cfg(test)]
mod tests {
    use zewif::{ProtocolAddress, sapling, transparent};

    use crate::zcashd_wallet::UfvkFingerprint;

    use super::{AddressId, AddressRegistry};

    #[test]
    fn test_address_id_from_protocol_address() {
        // Test transparent address
        let transparent =
            ProtocolAddress::Transparent(transparent::Address::new("t1abcdef".to_string()));
        let addr_id = AddressId::from_protocol_address(&transparent);
        assert!(matches!(addr_id, AddressId::Transparent(_)));
        assert_eq!(addr_id.protocol_type(), "transparent");

        // Test sapling address
        let shielded =
            ProtocolAddress::Sapling(Box::new(sapling::Address::new("zs1abcdef".to_string())));
        let addr_id = AddressId::from_protocol_address(&shielded);
        assert!(matches!(addr_id, AddressId::Sapling(_)));
        assert_eq!(addr_id.protocol_type(), "sapling");
    }

    #[test]
    fn test_address_id_from_string() {
        // Test transparent address
        let result = AddressId::from_address_string("t1WmEWuRKGcfi8iG3HxGNg3okswsdB54EXn");
        assert!(result.is_ok());
        let addr_id = result.unwrap();
        assert!(matches!(addr_id, AddressId::Transparent(_)));

        // Test sapling address
        let result = AddressId::from_address_string(
            "zs1uxklz44q04ttety3hke00we75lzy26wulmj5yu7qn6qxtqrmdq3l4222wuse24xs7mspwy8ddx0",
        );
        assert!(result.is_ok());
        let addr_id = result.unwrap();
        assert!(matches!(addr_id, AddressId::Sapling(_)));

        // Test unified address
        let result = AddressId::from_address_string(
            "u19mzuf4l37ny393m59v4mxx4t3uyxkh7qpqjdfvlfk9f504cv9w4fpl7cql0kqvssz8jay8mgl8lnrtvg6yzh9pranjj963acc3h2z2qt7007du0lsmdf862dyy40c3wmt0kq35k5z836tfljgzsqtdsccchayfjpygqzkx24l77ga3ngfgskqddyepz8we7ny4ggmt7q48cgvgu57mz",
        );
        assert!(result.is_ok());
        let addr_id = result.unwrap();
        assert!(matches!(addr_id, AddressId::Unified(_)));
    }

    #[test]
    fn test_address_id_display_and_fromstr() {
        // Test transparent address
        let addr_id = AddressId::Transparent("t1abcdef".to_string());
        let display_str = addr_id.to_string();
        assert_eq!(display_str, "t:t1abcdef");

        let parsed: AddressId = display_str.parse().unwrap();
        assert_eq!(parsed, addr_id);
    }

    #[test]
    fn test_address_registry() {
        let mut registry = AddressRegistry::new();

        // Create some test addresses and account IDs
        let addr1 = AddressId::Transparent("t1111".to_string());
        let addr2 = AddressId::Sapling("zs2222".to_string());
        let addr3 = AddressId::Unified("u1000".to_string());

        let mut bytes = [0u8; 32];
        let account1 = UfvkFingerprint::from_bytes(&bytes.clone()).unwrap();
        // Create a u256 value with just the first byte set to 1
        bytes[0] = 1;
        let account2 = UfvkFingerprint::from_bytes(&bytes).unwrap(); // Account ID 2

        // Register addresses to accounts
        registry.register(addr1.clone(), account1);
        registry.register(addr2.clone(), account1);
        registry.register(addr3.clone(), account2);

        // Test finding account for address
        assert_eq!(registry.find_account(&addr1), Some(&account1));
        assert_eq!(registry.find_account(&addr2), Some(&account1));
        assert_eq!(registry.find_account(&addr3), Some(&account2));

        // Test finding addresses for account
        let addrs_acct1 = registry.find_addresses_for_account(&account1);
        assert_eq!(addrs_acct1.len(), 2);
        assert!(addrs_acct1.contains(&&addr1));
        assert!(addrs_acct1.contains(&&addr2));

        let addrs_acct2 = registry.find_addresses_for_account(&account2);
        assert_eq!(addrs_acct2.len(), 1);

        // Test counts
        assert_eq!(registry.address_count(), 3);
        assert_eq!(registry.account_count(), 2);
    }
}
