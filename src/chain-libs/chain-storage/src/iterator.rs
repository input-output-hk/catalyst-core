use crate::{permanent_store::PermanentStore, BlockInfo, Error, Value};
use sled::Tree;

/// Iterator over blocks. Starts from n-th ancestor of the given block.
pub struct StorageIterator {
    state: IteratorState,
    from: Value,
    to: Value,
    block_info: Tree,
    blocks: Tree,
}

enum IteratorState {
    Permanent {
        iter: data_pile::SeqNoIter,
        current_length: u32,
    },
    Volatile {
        ids: Vec<Value>,
    },
}

impl StorageIterator {
    /// Create a new iterator. `from` and `to` values MUST have a path between
    /// them and this needs to be checked before the iterator is started.
    pub(crate) fn new(
        from: Value,
        from_length: u32,
        to: Value,
        permanent_store: PermanentStore,
        permanent_store_blocks: Tree,
        block_info: Tree,
        blocks: Tree,
    ) -> Result<Self, Error> {
        let state = if permanent_store_blocks.contains_key(from.as_ref())? {
            IteratorState::Permanent {
                iter: permanent_store.iter(from_length)?,
                current_length: from_length,
            }
        } else {
            IteratorState::Volatile {
                ids: gather_blocks_ids(from.clone(), to.clone(), &block_info, from_length)?,
            }
        };

        Ok(Self {
            state,
            from,
            to,
            block_info,
            blocks,
        })
    }
}

impl Iterator for StorageIterator {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.state {
            IteratorState::Permanent {
                iter,
                current_length,
            } => match iter.next() {
                Some(item) => {
                    *current_length += 1;
                    Some(Value::permanent(item))
                }
                None => {
                    self.state = IteratorState::Volatile {
                        ids: gather_blocks_ids(
                            self.from.clone(),
                            self.to.clone(),
                            &self.block_info,
                            *current_length,
                        )
                        .ok()?,
                    };
                    self.next()
                }
            },
            IteratorState::Volatile { ids } => {
                let id = ids.pop()?;
                self.blocks
                    .get(id.as_ref())
                    .ok()
                    .flatten()
                    .map(Value::volatile)
            }
        }
    }
}

fn gather_blocks_ids(
    from: Value,
    to: Value,
    block_info: &Tree,
    stop_at_length: u32,
) -> Result<Vec<Value>, Error> {
    let id_size = to.as_ref().len();
    let mut ids = Vec::new();

    let maybe_block_info = block_info.get(to.as_ref())?;

    let block_info_bin = match maybe_block_info {
        Some(block_info_bin) => block_info_bin,
        None => return Ok(ids),
    };

    let mut current_info = BlockInfo::deserialize(block_info_bin.as_ref(), id_size, to);

    loop {
        ids.push(current_info.id().clone());

        if current_info.id() == &from || current_info.chain_length() <= stop_at_length {
            break Ok(ids);
        }

        current_info = BlockInfo::deserialize(
            block_info
                .get(current_info.parent_id().as_ref())?
                .unwrap()
                .as_ref(),
            id_size,
            current_info.parent_id().clone(),
        );
    }
}
