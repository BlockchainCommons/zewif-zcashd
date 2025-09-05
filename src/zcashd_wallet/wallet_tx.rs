use anyhow::Result;
use std::collections::HashMap;
use zcash_primitives::transaction::Transaction;
use zewif::{BlockHash, Data};

use super::{
    CompactSize,
    orchard::OrchardTxMeta,
    sapling::SaplingNoteData,
    sprout::{JSOutPoint, SproutNoteData},
    transparent::SaplingOutPoint,
    u256,
};
use crate::{parse, parser::prelude::*};

#[derive(Debug, PartialEq)]
pub struct WalletTx {
    // CTransaction
    transaction: Transaction,

    // CMerkleTx
    hash_block: BlockHash,
    merkle_branch: Vec<u256>,
    index: i32,

    // CWalletTx
    map_value: HashMap<String, String>,
    map_sprout_note_data: HashMap<JSOutPoint, SproutNoteData>,
    order_form: Vec<(String, String)>,
    time_received_is_tx_time: i32,
    time_received: i32,
    is_from_me: bool,
    is_spent: bool,
    sapling_note_data: Option<HashMap<SaplingOutPoint, SaplingNoteData>>,
    orchard_tx_meta: Option<OrchardTxMeta>,

    unparsed_data: Data,
}

impl WalletTx {
    pub fn transaction(&self) -> &Transaction {
        &self.transaction
    }

    pub fn hash_block(&self) -> BlockHash {
        self.hash_block
    }

    pub fn merkle_branch(&self) -> &[u256] {
        &self.merkle_branch
    }

    pub fn index(&self) -> i32 {
        self.index
    }

    pub fn map_value(&self) -> &HashMap<String, String> {
        &self.map_value
    }

    pub fn map_sprout_note_data(&self) -> &HashMap<JSOutPoint, SproutNoteData> {
        &self.map_sprout_note_data
    }

    pub fn order_form(&self) -> &[(String, String)] {
        &self.order_form
    }

    pub fn time_received_is_tx_time(&self) -> i32 {
        self.time_received_is_tx_time
    }

    pub fn time_received(&self) -> i32 {
        self.time_received
    }

    pub fn is_from_me(&self) -> bool {
        self.is_from_me
    }

    pub fn is_spent(&self) -> bool {
        self.is_spent
    }

    pub fn sapling_note_data(&self) -> Option<&HashMap<SaplingOutPoint, SaplingNoteData>> {
        self.sapling_note_data.as_ref()
    }

    pub fn orchard_tx_meta(&self) -> Option<&OrchardTxMeta> {
        self.orchard_tx_meta.as_ref()
    }

    pub fn unparsed_data(&self) -> &Data {
        &self.unparsed_data
    }
}

struct ParseTransaction(zcash_primitives::transaction::Transaction);
impl Parse for ParseTransaction {
    fn parse(p: &mut Parser) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(ParseTransaction(
            zcash_primitives::transaction::Transaction::read(
                p,
                // The consensus branch ID that we use here will be ignored; it does not direct and is
                // not used by us after parsing; transaction serialization for v4 and below
                // transactions do not encode it and v5 transaction parsing ignores it entirely,
                // so it is essentially ephemeral as this parsing is only performed so that we can
                // reencode the transaction without the remainder of the `CMerkleTx` and `CWalletTx`
                // data.
                zcash_primitives::consensus::BranchId::Nu5,
            )?,
        ))
    }
}

impl Parse for WalletTx {
    fn parse(p: &mut Parser) -> Result<Self> {
        // CTransaction

        let ParseTransaction(transaction) = parse!(p, ParseTransaction, "wallet_transaction")?;

        // CMerkleTx
        let hash_block = parse!(p, "hash_block")?;
        let merkle_branch = parse!(p, "merkle_branch")?;
        let index = parse!(p, "index")?;

        // CWalletTx
        let unused_vt_prev = *parse!(p, CompactSize, "unused_vt_prev")?;
        assert!(
            unused_vt_prev == 0,
            "unused field in CWalletTx is not empty"
        );

        let map_value = parse!(p, "map_value")?;
        let map_sprout_note_data = parse!(p, "map_sprout_note_data")?;
        let order_form = parse!(p, "order_form")?;
        let time_received_is_tx_time = parse!(p, "time_received_is_tx_time")?;
        let time_received = parse!(p, "time_received")?;
        let from_me = parse!(p, "from_me")?;
        let is_spent = parse!(p, "is_spent")?;

        let mut sapling_note_data = None;
        if transaction.version().has_sapling() {
            let value = parse!(p, "sapling_note_data")?;
            sapling_note_data = Some(value);
        }

        let mut orchard_tx_meta: Option<OrchardTxMeta> = None;
        if transaction.version().has_orchard() {
            let value = parse!(p, "orchard_tx_meta")?;
            orchard_tx_meta = Some(value);
        }

        let unparsed_data = p.rest();
        if !unparsed_data.is_empty() {
            println!("💔 unparsed_data: {:?}", unparsed_data);
        }
        assert!(
            unparsed_data.is_empty(),
            "unparsed_data in CWalletTx is not empty"
        );

        Ok(Self {
            // CTransaction
            transaction,

            // CMerkleTx
            hash_block,
            merkle_branch,
            index,

            // CWalletTx
            map_value,
            map_sprout_note_data,
            order_form,
            time_received_is_tx_time,
            time_received,
            is_from_me: from_me,
            is_spent,
            sapling_note_data,
            orchard_tx_meta,

            unparsed_data,
        })
    }
}
