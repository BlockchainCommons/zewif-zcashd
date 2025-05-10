use crate::zcashd_wallet::{IncrementalWitness, u256};

/// The depth of the Sprout Merkle tree, set to 29 levels.
///
/// This constant defines the maximum depth of the Sprout note commitment tree,
/// which allows for 2^29 (over 500 million) note commitments to be included.
const INCREMENTAL_MERKLE_TREE_DEPTH: usize = 29;

/// A type alias for the SHA-256 compression function output used in Sprout Merkle trees.
///
/// The Sprout protocol uses a SHA-256 compression function for calculating node hashes
/// in its note commitment tree, which produces 256-bit (32-byte) hash values.
pub type SHA256Compress = u256;

/// A cryptographic witness proving that a Sprout note commitment exists in the note commitment tree.
///
/// `SproutWitness` is a specialized form of incremental Merkle tree witness for the
/// original Sprout shielded protocol. It proves that a specific note commitment is
/// included in the global Sprout note commitment tree, which is necessary when spending
/// a Sprout note.
///
/// # Zcash Concept Relation
/// In Zcash's original Sprout protocol (the first shielded protocol):
///
/// - **Note Commitment Tree**: A 29-level Merkle tree containing all Sprout note commitments
/// - **Merkle Path**: The path from a leaf (note commitment) to the root of the tree
/// - **Witness**: The authentication path proving a leaf exists in the tree
/// - **Anchors**: Root hashes of the note commitment tree at specific blockchain heights
///
/// When spending a Sprout note, a zero-knowledge proof must demonstrate that the
/// note exists in the tree at a specific anchor, without revealing which specific
/// commitment is being spent. The witness provides the necessary path information.
///
/// # Data Preservation
/// During wallet migration, complete witness data must be preserved for all unspent
/// Sprout notes. This includes:
///
/// - The authentication path (sequence of hashes forming the Merkle path)
/// - The position of the note commitment in the tree
/// - The tree depth (29 for Sprout)
///
/// Without this witness data, unspent Sprout notes cannot be spent as it would be
/// impossible to prove their inclusion in the note commitment tree.
///
/// # Implementation Details
/// This type is an alias for `IncrementalWitness<29, SHA256Compress>`, representing a
/// witness for a Merkle tree with 29 levels using SHA-256 compression as the hash function.
pub type SproutWitness = IncrementalWitness<INCREMENTAL_MERKLE_TREE_DEPTH, SHA256Compress>;
