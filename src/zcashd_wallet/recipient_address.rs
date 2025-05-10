use anyhow::Result;

use super::{
    ReceiverType, orchard::OrchardRawAddress, sapling::SaplingZPaymentAddress,
    transparent::{KeyId, ScriptId},
};
use crate::{parse, parser::prelude::*};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RecipientAddress {
    KeyId(KeyId),
    ScriptId(ScriptId),
    Sapling(SaplingZPaymentAddress),
    Orchard(OrchardRawAddress),
}

impl Parse for RecipientAddress {
    fn parse(parser: &mut Parser) -> Result<Self>
    where
        Self: Sized,
    {
        let receiver_type = parse!(parser, ReceiverType, "receiver_type")?;
        let result = match receiver_type {
            ReceiverType::P2PKH => {
                let key_id = parse!(parser, KeyId, "key_id")?;
                RecipientAddress::KeyId(key_id)
            }
            ReceiverType::P2SH => {
                let script_id = parse!(parser, ScriptId, "script_id")?;
                RecipientAddress::ScriptId(script_id)
            }
            ReceiverType::Sapling => {
                let sapling_z_payment_address =
                    parse!(parser, SaplingZPaymentAddress, "sapling_z_payment_address")?;
                RecipientAddress::Sapling(sapling_z_payment_address)
            }
            ReceiverType::Orchard => {
                let orchard_raw_address = parse!(parser, OrchardRawAddress, "orchard_raw_address")?;
                RecipientAddress::Orchard(orchard_raw_address)
            }
        };
        Ok(result)
    }
}
