use anyhow::Result;

use crate::{parse, parser::prelude::*, zcashd_wallet::u256};

/// An efficient incremental Merkle tree implementation for note commitments in Zcash.
///
/// `IncrementalMerkleTree` stores a partial Merkle tree structure that can be efficiently
/// updated as new note commitments are added to the tree. Rather than storing the entire tree,
/// it maintains only the minimum necessary nodes to reconstruct the current tree state.
///
/// # Zcash Concept Relation
/// In Zcash's shielded protocols, all note commitments are stored in Merkle trees:
///
/// - **Commitment Trees**: Store cryptographic commitments to shielded notes
/// - **Tree Roots (Anchors)**: Used in zero-knowledge proofs to validate spent notes
/// - **Authentication Paths**: Prove that a specific note exists in the tree
///
/// Different protocols use different tree depths and hash functions:
/// - Sprout: 29-level trees with SHA-256 compression
/// - Sapling: 32-level trees with Pedersen hashes
/// - Orchard: 32-level trees with Poseidon hashes
///
/// # Data Preservation
/// During wallet migration, the incremental Merkle tree structure must be preserved
/// to maintain the ability to:
///
/// - Generate witnesses for spending notes in the future
/// - Validate that notes have been included in the blockchain
/// - Continue appending new notes to the tree
///
/// The tree structure is kept minimal by storing only the necessary nodes at each level.
///
/// # Examples
/// ```
/// # use zewif_zcashd::zcashd_wallet::{IncrementalMerkleTree, u256};
/// // Create a new empty incremental Merkle tree
/// let mut tree = IncrementalMerkleTree::new();
///
/// // Add a leaf node to the left position
/// let leaf_hash = u256::default(); // In practice, this would be a note commitment hash
/// tree.set_left(leaf_hash);
///
/// // The tree can be used to generate witnesses for spending notes
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IncrementalMerkleTree {
    /// The left child at the current insertion point (None if empty)
    left: Option<u256>,

    /// The right child at the current insertion point (None if empty)
    right: Option<u256>,

    /// Parent hashes at each level above the current insertion point
    parents: Vec<Option<u256>>,
}

impl IncrementalMerkleTree {
    /// Creates a new empty incremental Merkle tree.
    ///
    /// This initializes an empty tree with no nodes at any level.
    ///
    /// # Examples
    /// ```
    /// # use zewif_zcashd::zcashd_wallet::IncrementalMerkleTree;
    /// let tree = IncrementalMerkleTree::new();
    /// assert!(tree.left().is_none());
    /// assert!(tree.right().is_none());
    /// assert!(tree.parents().is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            left: None,
            right: None,
            parents: Vec::new(),
        }
    }

    /// Creates an incremental Merkle tree with specified field values.
    ///
    /// This constructor allows directly setting the tree's internal state,
    /// which is useful when deserializing from storage or creating a specific
    /// tree state for testing.
    ///
    /// # Arguments
    /// * `left` - The left child at the current insertion point
    /// * `right` - The right child at the current insertion point
    /// * `parents` - The parent nodes at each level above the current insertion point
    ///
    /// # Examples
    /// ```
    /// # use zewif_zcashd::zcashd_wallet::{IncrementalMerkleTree, u256};
    /// // Create a tree with specific values
    /// let left = Some(u256::default());
    /// let right = Some(u256::default());
    /// let parents = vec![Some(u256::default()), None];
    ///
    /// let tree = IncrementalMerkleTree::with_fields(left, right, parents);
    /// ```
    pub fn with_fields(
        left: Option<u256>,
        right: Option<u256>,
        parents: Vec<Option<u256>>,
    ) -> Self {
        Self {
            left,
            right,
            parents,
        }
    }

    /// Returns the left child at the current insertion point.
    ///
    /// # Returns
    /// * `Some(u256)` if the left child exists
    /// * `None` if the left position is empty
    pub fn left(&self) -> Option<u256> {
        self.left
    }

    /// Sets the left child at the current insertion point.
    ///
    /// # Arguments
    /// * `left` - The hash value to set as the left child
    pub fn set_left(&mut self, left: u256) {
        self.left = Some(left);
    }

    /// Returns the right child at the current insertion point.
    ///
    /// # Returns
    /// * `Some(u256)` if the right child exists
    /// * `None` if the right position is empty
    pub fn right(&self) -> Option<u256> {
        self.right
    }

    /// Sets the right child at the current insertion point.
    ///
    /// # Arguments
    /// * `right` - The hash value to set as the right child
    pub fn set_right(&mut self, right: u256) {
        self.right = Some(right);
    }

    /// Returns a reference to the parent nodes at each level above the current insertion point.
    ///
    /// Each entry in the vector represents a level in the tree, with `Some(u256)` indicating
    /// a populated node and `None` indicating an empty position.
    ///
    /// # Returns
    /// A reference to the vector of parent nodes
    pub fn parents(&self) -> &Vec<Option<u256>> {
        &self.parents
    }

    /// Adds a parent node to the next level of the tree.
    ///
    /// This method is typically used when building or updating the tree structure.
    ///
    /// # Arguments
    /// * `parent` - The hash value to add as a parent, or None if the position is empty
    pub fn push_parent(&mut self, parent: Option<u256>) {
        self.parents.push(parent);
    }
}

/// Default implementation creates an empty incremental Merkle tree
impl Default for IncrementalMerkleTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of the Parse trait for binary deserialization
impl Parse for IncrementalMerkleTree {
    fn parse(p: &mut Parser) -> Result<Self> {
        let left = parse!(p, "left")?;
        let right = parse!(p, "right")?;
        let parents = parse!(p, "parents")?;
        Ok(Self::with_fields(left, right, parents))
    }
}
