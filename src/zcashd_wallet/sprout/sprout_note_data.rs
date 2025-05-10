use anyhow::Result;

use crate::{parse, parser::prelude::*, zcashd_wallet::u256};

use super::{SproutPaymentAddress, SproutWitness};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SproutNoteData {
    address: SproutPaymentAddress,
    nullifer: Option<u256>,
    witnesses: Vec<SproutWitness>,
    witness_height: i32,
}

impl SproutNoteData {
    pub fn address(&self) -> SproutPaymentAddress {
        self.address.clone()
    }

    pub fn nullifer(&self) -> Option<u256> {
        self.nullifer
    }

    pub fn witnesses(&self) -> &Vec<SproutWitness> {
        &self.witnesses
    }

    pub fn witness_height(&self) -> i32 {
        self.witness_height
    }
}

impl Parse for SproutNoteData {
    fn parse(p: &mut Parser) -> Result<Self> {
        Ok(Self {
            address: parse!(p, "address")?,
            nullifer: parse!(p, "nullifer")?,
            witnesses: parse!(p, "witnesses")?,
            witness_height: parse!(p, "witness_height")?,
        })
    }
}
