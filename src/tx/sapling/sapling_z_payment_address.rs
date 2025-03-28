use anyhow::Result;
use zcash_address::{ToAddress, ZcashAddress};

use zewif::{Blob, Network};
use zewif::{parse, parser::prelude::*};

use crate::zewif_network_to_zcash_address_network;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SaplingZPaymentAddress {
    diversifier: Blob<11>,
    pk: Blob<32>,
}

impl SaplingZPaymentAddress {
    pub fn to_string(&self, network: Network) -> String {
        // Concatenate diversifier (11 bytes) and pk (32 bytes) into a 43-byte array
        let mut bytes = [0u8; 43];
        bytes[..11].copy_from_slice(self.diversifier.as_slice());
        bytes[11..].copy_from_slice(self.pk.as_slice());
        let addr =
            ZcashAddress::from_sapling(zewif_network_to_zcash_address_network(network), bytes);
        addr.to_string()
    }

    pub fn diversifier(&self) -> &Blob<11> {
        &self.diversifier
    }

    pub fn pk(&self) -> &Blob<32> {
        &self.pk
    }
}

impl Parse for SaplingZPaymentAddress {
    fn parse(p: &mut Parser) -> Result<Self> {
        let diversifier = parse!(p, "diversifier")?;
        let pk = parse!(p, "pk")?;
        Ok(SaplingZPaymentAddress { diversifier, pk })
    }
}
