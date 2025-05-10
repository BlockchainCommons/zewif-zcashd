use anyhow::Result;
use ::orchard::keys::IncomingViewingKey;
use std::collections::HashMap;

use crate::{parse, parser::prelude::*, zcashd_wallet::ClientVersion};

#[derive(Debug, Clone, PartialEq)]
pub struct OrchardTxMeta {
    version: ClientVersion,
    receiving_keys: HashMap<u32, IncomingViewingKey>,
    actions_spending_my_nodes: Vec<u32>,
}

impl OrchardTxMeta {
    /// Returns the client version
    pub fn version(&self) -> ClientVersion {
        self.version
    }

    /// Returns the IVK that received the output at the given action index, if any.
    pub fn receiving_key(&self, index: u32) -> Option<&IncomingViewingKey> {
        self.receiving_keys.get(&index)
    }

    /// Returns the entire action data map
    pub fn receiving_keys(&self) -> &HashMap<u32, IncomingViewingKey> {
        &self.receiving_keys
    }

    /// Returns the list of actions spending nodes owned by this wallet
    pub fn actions_spending_my_nodes(&self) -> &[u32] {
        &self.actions_spending_my_nodes
    }
}

impl Parse for OrchardTxMeta {
    fn parse(parser: &mut Parser) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            version: parse!(parser, "version")?,
            receiving_keys: parse!(parser, "receiving_keys")?,
            actions_spending_my_nodes: parse!(parser, "actions_spending_my_nodes")?,
        })
    }
}
