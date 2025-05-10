use anyhow::Result;

use crate::{parse, parser::prelude::*, zcashd_wallet::u256};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SproutPaymentAddress {
    a_pk: u256,
    pk_enc: u256,
}

impl SproutPaymentAddress {
    pub fn a_pk(&self) -> u256 {
        self.a_pk
    }

    pub fn pk_enc(&self) -> u256 {
        self.pk_enc
    }
}

impl Parse for SproutPaymentAddress {
    fn parse(p: &mut Parser) -> Result<Self> {
        Ok(Self {
            a_pk: parse!(p, "sprout payment address a_pk")?,
            pk_enc: parse!(p, "sprout payment address pk_enc")?,
        })
    }
}
