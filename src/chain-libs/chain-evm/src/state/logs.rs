use evm::backend::Log;

use crate::machine::BlockHash;

use super::Trie;

/// In-memory representation of all logs.
pub type BlockLogsTrie = Trie<BlockHash, Vec<Log>>;

#[derive(Default, Clone, PartialEq, Eq)]
pub struct LogsState {
    block_logs: BlockLogsTrie,
}

impl LogsState {
    pub fn put(&mut self, block_hash: BlockHash, logs: Vec<Log>) {
        self.block_logs = self.block_logs.clone().put(block_hash, logs);
    }
}
