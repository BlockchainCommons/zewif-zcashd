use anyhow::Result;
use zcash_address::{ToAddress, ZcashAddress, unified::Encoding};
use zewif::{Blob, Blob32, Network};

use crate::{parse, parser::prelude::*};

use crate::migrate::primitives::address_network_from_zewif;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrchardRawAddress {
    diversifier: Blob<11>,
    pk_d: Blob32,
}

impl OrchardRawAddress {
    pub fn diversifier(&self) -> &Blob<11> {
        &self.diversifier
    }

    pub fn pk_d(&self) -> &Blob32 {
        &self.pk_d
    }
}

impl OrchardRawAddress {
    pub fn to_string(&self, network: Network) -> String {
        // Concatenate diversifier (11 bytes) and pk_d (32 bytes) into a 43-byte array
        let mut bytes = [0u8; 43];
        bytes[..11].copy_from_slice(self.diversifier.as_slice());
        bytes[11..].copy_from_slice(self.pk_d.as_slice());

        // Create an Orchard receiver
        let orchard_receiver = zcash_address::unified::Receiver::Orchard(bytes);

        // Create a Unified Address with just this receiver
        let unified_addr = zcash_address::unified::Address::try_from_items(vec![orchard_receiver])
            .expect("A single valid receiver should create a valid unified address");

        // Create a ZcashAddress from the unified address
        let addr = ZcashAddress::from_unified(address_network_from_zewif(network), unified_addr);
        addr.to_string()
    }
}

impl Parse for OrchardRawAddress {
    fn parse(p: &mut Parser) -> Result<Self> {
        let diversifier = parse!(p, "diversifier")?;
        let pk_d = parse!(p, "pk_d")?;
        Ok(OrchardRawAddress { diversifier, pk_d })
    }
}
