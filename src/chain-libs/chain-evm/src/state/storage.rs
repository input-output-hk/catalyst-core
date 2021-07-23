use crate::state::trie::Trie;

use primitive_types::U256;

/// Representation of a storage key. 256-bit big-endian integer.
pub type Key = U256;
/// Representation of a storage key. 256-bit big-endian integer.
pub type Value = U256;
/// In-memory representation of account storage.
pub type Storage = Trie<Key, Value>;
