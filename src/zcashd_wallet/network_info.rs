use anyhow::Result;
use zewif::Network;

use crate::{parse, parser::prelude::*};

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkInfo {
    zcash: String,
    network: Network,
}

impl NetworkInfo {
    pub fn zcash(&self) -> &str {
        &self.zcash
    }

    pub fn network(&self) -> Network {
        self.network
    }

    pub fn to_address_encoding_network(&self) -> zcash_protocol::consensus::Network {
        use zcash_protocol::consensus::Network::*;
        match self.network {
            Network::Main => MainNetwork,
            Network::Test => TestNetwork,
            // Regtest addresses are encoded as for the test network.
            Network::Regtest => TestNetwork,
        }
    }
}

impl Parse for NetworkInfo {
    fn parse(p: &mut Parser) -> Result<Self> {
        let (zcash, identifier): (String, String) = parse!(p, "(zcash, identifier)")?;
        let network = Network::try_from(identifier)?;
        Ok(Self { zcash, network })
    }
}
