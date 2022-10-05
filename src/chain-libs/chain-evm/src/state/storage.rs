use crate::state::trie::Trie;

use ethereum_types::H256;

/// Representation of a storage key. Fixed-size uninterpreted hash type with 32 bytes (256 bits) size.
pub type Key = H256;
/// Representation of a storage key. Fixed-size uninterpreted hash type with 32 bytes (256 bits) size.
pub type Value = H256;
/// In-memory representation of account storage.
pub type Storage = Trie<Key, Value>;
