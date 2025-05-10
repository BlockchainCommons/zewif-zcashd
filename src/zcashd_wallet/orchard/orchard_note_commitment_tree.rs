use anyhow::Result;
use bridgetree::{BridgeTree, Position};
use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    collections::BTreeMap,
    io::{self, Read},
};

use ::orchard::tree::MerkleHashOrchard;
use zcash_encoding::{Optional, Vector};
use zcash_primitives::{consensus::BlockHeight, merkle_tree::read_position, transaction::TxId};

use super::bridgetree_parsing::read_tree;
use crate::parser::prelude::*;

// Constants for tree validation
const ORCHARD_TREE_DEPTH: u8 = 32;

/// A data structure holding chain positions for a single transaction.
#[derive(Clone, Debug)]
struct NotePositions {
    /// The height of the block containing the transaction.
    tx_height: BlockHeight,
    /// A map from the index of an Orchard action tracked by this wallet, to the position
    /// of the output note's commitment within the global Merkle tree.
    note_positions: BTreeMap<u32, incrementalmerkletree::Position>,
}

/// Represents the complete Orchard note commitment tree
#[derive(Debug, Clone)]
pub struct OrchardNoteCommitmentTree {
    last_checkpoint: Option<BlockHeight>,
    commitment_tree: BridgeTree<MerkleHashOrchard, BlockHeight, ORCHARD_TREE_DEPTH>,
    note_positions: Vec<(TxId, NotePositions)>,
}

impl OrchardNoteCommitmentTree {
    const NOTE_STATE_V1: u8 = 1;

    fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        match reader.read_u8()? {
            Self::NOTE_STATE_V1 => {
                let last_checkpoint = Optional::read(&mut reader, |r| {
                    r.read_u32::<LittleEndian>().map(BlockHeight::from)
                })?;

                let commitment_tree = read_tree(&mut reader)?;

                // Read note positions.
                let note_positions: Vec<(TxId, NotePositions)> =
                    Vector::read_collected(&mut reader, |mut r| {
                        Ok((
                            TxId::read(&mut r)?,
                            NotePositions {
                                tx_height: r.read_u32::<LittleEndian>().map(BlockHeight::from)?,
                                note_positions: Vector::read_collected(r, |r| {
                                    Ok((r.read_u32::<LittleEndian>()?, read_position(r)?))
                                })?,
                            },
                        ))
                    })?;

                Ok(Self {
                    last_checkpoint,
                    commitment_tree,
                    note_positions,
                })
            }
            unrecognized => Err(io::Error::other(format!(
                "Unrecognized Orchard note position serialization version: {}",
                unrecognized
            ))),
        }
    }

    /// Convert to Zewif IncremetalWitness format
    fn extract_witness(
        &self,
        _position: Position,
    ) -> zewif::IncrementalWitness<32, MerkleHashOrchard> {
        todo!()
    }
}

impl Parse for OrchardNoteCommitmentTree {
    fn parse(p: &mut Parser) -> Result<Self> {
        Ok(OrchardNoteCommitmentTree::read(p)?)
    }
}
