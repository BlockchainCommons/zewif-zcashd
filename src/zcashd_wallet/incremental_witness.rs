use anyhow::Result;

use super::IncrementalMerkleTree;
use crate::{parse, parser::prelude::*};

/// An authentication path generator for a specific note in a Merkle tree.
///
/// `IncrementalWitness` creates and maintains the cryptographic evidence needed to prove
/// that a specific note commitment exists in the blockchain's commitment tree. It can be
/// efficiently updated as new notes are added to the tree, without requiring a complete
/// tree rebuild.
///
/// # Zcash Concept Relation
/// In Zcash's shielded protocols, witnesses serve several critical functions:
///
/// - **Spending Notes**: When spending a shielded note, the witness proves the note exists
///   in the blockchain's commitment tree without revealing which specific note is being spent
///
/// - **Authentication Paths**: A witness contains the minimum set of hash values needed to
///   verify that a specific leaf (note commitment) is part of a tree with a known root (anchor)
///
/// - **Zero-Knowledge Proofs**: The witness data is used within zk-SNARKs to validate
///   transactions without revealing private information
///
/// Each shielded protocol has a different Merkle tree depth:
/// - Sprout: 29 levels deep, using SHA-256 compression
/// - Sapling: 32 levels deep, using Pedersen hashes
/// - Orchard: 32 levels deep, using Poseidon hashes
///
/// # Data Preservation
/// During wallet migration, maintaining complete witness data is absolutely critical:
///
/// - **Unspent Notes**: Without valid witnesses, unspent notes cannot be spent after migration
/// - **Path Components**: The authentication path for each note must be preserved exactly
/// - **Tree State**: The current state of the tree at the time of the witness creation
///
/// # Type Parameters
/// * `DEPTH` - The depth of the Merkle tree (29 for Sprout, 32 for Sapling/Orchard)
/// * `Hash` - The hash type used for tree nodes (varies by protocol)
///
/// # Examples
/// ```
/// # use zewif::Blob;
/// # use zewif_zcashd::zcashd_wallet::{IncrementalMerkleTree, IncrementalWitness};
/// // Create a tree with a specific structure
/// let tree = IncrementalMerkleTree::new();
///
/// // Create a witness for a note at a specific position
/// let filled: Vec<Blob<32>> = Vec::new();
/// let cursor = None;
/// let witness = IncrementalWitness::<32, Blob<32>>::with_fields(tree, filled, cursor);
///
/// // In a real implementation, this witness would be updated as new notes are added to the tree
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IncrementalWitness<const DEPTH: usize, Hash> {
    /// The Merkle tree as it was when the witness was created
    tree: IncrementalMerkleTree,

    /// Hashes filled in since the witness was created
    filled: Vec<Hash>,

    /// Optional cursor for tracking the witness position
    cursor: Option<IncrementalMerkleTree>,
}

impl<const DEPTH: usize, Hash> IncrementalWitness<DEPTH, Hash> {
    /// Creates an incremental witness with specified field values.
    ///
    /// This constructor allows directly setting the witness's internal state,
    /// which is useful when deserializing from storage or creating a specific
    /// witness state.
    ///
    /// # Arguments
    /// * `tree` - The Merkle tree state when the witness was created
    /// * `filled` - Hashes of nodes filled since witness creation
    /// * `cursor` - Optional cursor for tracking position updates
    ///
    /// # Returns
    /// A new `IncrementalWitness` with the specified field values
    pub fn with_fields(
        tree: IncrementalMerkleTree,
        filled: Vec<Hash>,
        cursor: Option<IncrementalMerkleTree>,
    ) -> Self {
        Self {
            tree,
            filled,
            cursor,
        }
    }

    /// Returns a reference to the Merkle tree state when the witness was created.
    ///
    /// This represents the base state of the tree that the witness is built upon,
    /// before any additional nodes were filled in.
    ///
    /// # Returns
    /// A reference to the original Merkle tree state
    pub fn tree(&self) -> &IncrementalMerkleTree {
        &self.tree
    }

    /// Returns a reference to the hashes filled in since the witness was created.
    ///
    /// As new notes are added to the tree, their hashes are added to this vector
    /// to keep the witness up to date with the current tree state.
    ///
    /// # Returns
    /// A reference to the vector of filled hash values
    pub fn filled(&self) -> &Vec<Hash> {
        &self.filled
    }

    /// Returns a reference to the optional cursor tracking the witness position.
    ///
    /// The cursor is used to efficiently update the witness as the tree changes,
    /// tracking the current position for appending new nodes.
    ///
    /// # Returns
    /// A reference to the optional cursor
    pub fn cursor(&self) -> &Option<IncrementalMerkleTree> {
        &self.cursor
    }
}

/// Implementation of the Parse trait for binary deserialization
impl<const DEPTH: usize, Hash: Parse> Parse for IncrementalWitness<DEPTH, Hash> {
    fn parse(p: &mut Parser) -> Result<Self> {
        let tree = parse!(p, "tree")?;
        let filled = parse!(p, "filled")?;
        let cursor = parse!(p, "cursor")?;
        Ok(Self::with_fields(tree, filled, cursor))
    }
}
