use anyhow::Result;
use zewif::{Blob, sapling::SaplingIncomingViewingKey};
use crate::{parse, parser::prelude::*, zcashd_wallet::IncrementalWitness};

pub type SaplingWitness = IncrementalWitness<32, Blob<32>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SaplingNoteData {
    version: i32,
    incoming_viewing_key: SaplingIncomingViewingKey,
    nullifier: Option<Blob<32>>,
    witnesses: Vec<SaplingWitness>,
    witness_height: i32,
}

impl SaplingNoteData {
    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn incoming_viewing_key(&self) -> &SaplingIncomingViewingKey {
        &self.incoming_viewing_key
    }

    pub fn nullifier(&self) -> Option<&Blob<32>> {
        self.nullifier.as_ref()
    }

    pub fn witnesses(&self) -> &[SaplingWitness] {
        &self.witnesses
    }

    pub fn witness_height(&self) -> i32 {
        self.witness_height
    }
}

impl Parse for SaplingNoteData {
    fn parse(p: &mut Parser) -> Result<Self> {
        Ok(Self {
            version: parse!(p, "version")?,
            incoming_viewing_key: parse!(p, "incoming_viewing_key")?,
            nullifier: parse!(p, "nullifier")?,
            witnesses: parse!(p, "witnesses")?,
            witness_height: parse!(p, "witness_height")?,
        })
    }
}
