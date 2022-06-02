mod handlers;
mod routes;

pub use routes::{filter, update_filter};

use jormungandr_lib::{crypto::account::Identifier, interfaces::Value};
use sled::{IVec, Transactional};
use std::mem::size_of;
use voting_hir::VoterHIR;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DbError(#[from] sled::Error),

    #[error(transparent)]
    DbTxError(#[from] sled::transaction::TransactionError),

    #[error("internal error")]
    InternalError,
}

pub type Tag = String;
type Group = String;

#[repr(transparent)]
struct TagId(u32);

impl TagId {
    const MIN: Self = Self(u32::MIN);

    fn from_be_bytes(bytes: &[u8]) -> Result<Self, Error> {
        bytes
            .try_into()
            .map_err(|_| Error::InternalError)
            .map(u32::from_be_bytes)
            .map(Self)
    }

    fn to_be_bytes(&self) -> [u8; 4] {
        self.0.to_be_bytes()
    }

    fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Clone)]
pub struct SharedContext {
    _db: sled::Db,
    tags: sled::Tree,
    entries: sled::Tree,
}

impl SharedContext {
    fn new(db: sled::Db) -> Result<Self, Error> {
        let tags = db.open_tree("tags")?;
        let entries = db.open_tree("entries")?;

        Ok(Self {
            _db: db,
            tags,
            entries,
        })
    }

    #[tracing::instrument(skip(self))]
    pub fn get_voting_power(
        &self,
        tag: &str,
        id: &Identifier,
    ) -> Result<Option<Vec<(Group, Value)>>, Error> {
        let tag = if let Some(tag) = self.tags.get(tag)? {
            tag
        } else {
            return Ok(None);
        };

        // the fixed part of the key, tag + user, not including the group (or using the empty
        // group, which is the min, depending on the point of view).
        let key_prefix = {
            let mut key = [0u8; size_of::<TagId>() + 32usize];

            let (tag_part, id_part) = key.split_at_mut(size_of::<TagId>());
            tag_part.copy_from_slice(&*tag);
            id_part.copy_from_slice(id.as_ref().as_ref());

            key
        };

        let mut result: Vec<(Group, Value)> = vec![];

        for entries in self.entries.range(key_prefix..) {
            let (k, v) = entries?;

            // we are using only a prefix of the actual key, so we want to compare that part only
            if k[0..key_prefix.len()] > key_prefix[..] {
                break;
            }

            let group = String::from_utf8(k[key_prefix.len()..].to_vec())
                .map_err(|_| Error::InternalError)?;

            let voting_power = Value::from(u64::from_be_bytes(v.as_ref().try_into().unwrap()));

            result.push((group, voting_power));
        }

        Ok(Some(result))
    }

    pub fn get_tags(&self) -> Result<Vec<Tag>, Error> {
        let mut result = vec![];
        for entries in self.tags.iter() {
            let (tag, _) = entries?;
            result.push(String::from_utf8(tag.to_vec()).map_err(|_| Error::InternalError)?);
        }

        Ok(result)
    }
}

// do NOT implement/derive Clone for this. The implementation of update relies on &mut self and the
// split in a reader type and a writer type is to enforce a single writer.
pub struct UpdateHandle {
    _db: sled::Db,
    tags: sled::Tree,
    entries: sled::Tree,
    seqs: sled::Tree,
}

const TAG_SEQ_KEY: &str = "TID";

impl UpdateHandle {
    fn new(db: sled::Db) -> Result<Self, Error> {
        let tags = db.open_tree("tags")?;
        let entries = db.open_tree("entries")?;
        let seqs = db.open_tree("seqs")?;

        if seqs.get(TAG_SEQ_KEY)?.is_none() {
            seqs.insert(TAG_SEQ_KEY, &TagId::MIN.to_be_bytes())?;
        }

        Ok(UpdateHandle {
            _db: db,
            tags,
            entries,
            seqs,
        })
    }

    #[tracing::instrument(skip(self, snapshot))]
    pub async fn update(
        &mut self,
        tag: &str,
        snapshot: impl IntoIterator<Item = VoterHIR>,
    ) -> Result<(), Error> {
        let mut batch = sled::Batch::default();

        enum Tag {
            Existing(IVec),
            New(IVec),
        }

        let tag_id = if let Some(existing) = self.tags.get(tag)? {
            // remove all existing entries for this tag so the ones that are not present in the new
            // input get deleted
            for entry in self.entries.range(&*existing..) {
                let (k, _) = entry?;

                // `existing` here is a prefix of the tree's key, since we are going to remove
                // everything that starts with this tag, we don't need neither the public key nor
                // the group.
                //
                // this is also equivalent to looping in the range(existing..existing+1).
                if k[0..existing.len()] > *existing {
                    break;
                }

                // notice that this uses the same Batch as the inserts, so if the entry exists in
                // `snapshot` then it will not incur in a delete followed by an insert to the db.
                batch.remove(k);
            }

            Tag::Existing(existing)
        } else {
            // unwrapping here is fine because the constructor initializes this entry to 0
            Tag::New(self.seqs.get(TAG_SEQ_KEY)?.unwrap())
        };

        for entry in snapshot.into_iter() {
            let VoterHIR {
                voting_key,
                voting_group,
                voting_power,
            } = entry;

            let voting_key_bytes = voting_key.as_ref().as_ref();

            let mut key = Vec::with_capacity(
                size_of::<TagId>() + voting_key_bytes.len() + voting_group.as_bytes().len(),
            );

            match &tag_id {
                Tag::Existing(tag_id) | Tag::New(tag_id) => key.extend(&**tag_id),
            }

            key.extend(voting_key_bytes);
            key.extend(voting_group.as_bytes());

            batch.insert(key, &u64::from(voting_power).to_be_bytes());
        }

        {
            let tag = tag.to_string();
            let tags = self.tags.clone();
            let entries = self.entries.clone();
            let seqs = self.seqs.clone();

            tokio::task::spawn_blocking(move || {
                (&tags, &entries, &seqs).transaction(move |(tags, entries, seqs)| {
                    if let Tag::New(id) = &tag_id {
                        tags.insert(tag.as_bytes(), id)?;
                        seqs.insert(
                            TAG_SEQ_KEY,
                            &TagId::from_be_bytes(id.as_ref())
                                .unwrap()
                                .next()
                                .to_be_bytes(),
                        )?;
                    }

                    entries.apply_batch(&batch)?;

                    Ok(())
                })?;

                Ok(())
            })
            .await
            .unwrap()
            .map_err(Error::DbTxError)?;
        }

        Ok(())
    }
}

pub fn new_context() -> Result<(SharedContext, UpdateHandle), Error> {
    let db = sled::Config::new().temporary(true).open()?;

    Ok((SharedContext::new(db.clone())?, UpdateHandle::new(db)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    pub async fn test_snapshot() {
        let (rx, mut tx) = new_context().unwrap();

        let keys = [
            Identifier::from_hex(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            Identifier::from_hex(
                "1111111111111111111111111111111111111111111111111111111111111111",
            )
            .unwrap(),
        ];

        const GROUP1: &str = "group1";
        const GROUP2: &str = "group2";

        const TAG1: &str = "tag1";
        const TAG2: &str = "tag2";

        let key_0_values = [
            (GROUP1.to_string(), Value::from(1)),
            (GROUP2.to_string(), Value::from(2)),
        ];

        let content_a = std::iter::repeat(keys[0].clone())
            .take(key_0_values.len())
            .zip(key_0_values.iter().cloned())
            .map(|(voting_key, (voting_group, voting_power))| VoterHIR {
                voting_key,
                voting_group,
                voting_power,
            })
            .collect::<Vec<_>>();

        tx.update(TAG1, content_a.clone()).await.unwrap();

        let key_1_values = [(GROUP1.to_string(), Value::from(3))];

        let content_b = std::iter::repeat(keys[1].clone())
            .take(key_1_values.len())
            .zip(key_1_values.iter().cloned())
            .map(|(voting_key, (voting_group, voting_power))| VoterHIR {
                voting_key,
                voting_group,
                voting_power,
            })
            .collect::<Vec<_>>();

        tx.update(TAG2, [content_a, content_b].concat())
            .await
            .unwrap();

        assert_eq!(
            &key_0_values[..],
            &rx.get_voting_power(TAG1, &keys[0]).unwrap().unwrap()[..],
        );

        assert!(&rx
            .get_voting_power(TAG1, &keys[1])
            .unwrap()
            .unwrap()
            .is_empty(),);

        assert_eq!(
            &key_1_values[..],
            &rx.get_voting_power(TAG2, &keys[1]).unwrap().unwrap()[..],
        );
    }

    #[tokio::test]
    pub async fn test_snapshot_previous_entries_get_deleted() {
        const TAG1: &str = "tag1";
        const TAG2: &str = "tag2";

        let (rx, mut tx) = new_context().unwrap();

        let voting_key = Identifier::from_hex(
            "0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        let inputs = [
            VoterHIR {
                voting_key: voting_key.clone(),
                voting_group: "GROUP1".into(),
                voting_power: 1.into(),
            },
            VoterHIR {
                voting_key: voting_key.clone(),
                voting_group: "GROUP2".into(),
                voting_power: 1.into(),
            },
        ];

        tx.update(TAG1, inputs.clone()).await.unwrap();
        tx.update(TAG2, inputs.clone()).await.unwrap();

        assert_eq!(
            rx.get_voting_power(TAG1, &voting_key).unwrap().unwrap(),
            inputs
                .iter()
                .cloned()
                .map(|hir| (hir.voting_group, hir.voting_power))
                .collect::<Vec<_>>()
        );

        tx.update(TAG1, inputs[0..1].to_vec()).await.unwrap();

        assert_eq!(
            rx.get_voting_power(TAG1, &voting_key).unwrap().unwrap(),
            inputs[0..1]
                .iter()
                .cloned()
                .map(|hir| (hir.voting_group, hir.voting_power))
                .collect::<Vec<_>>()
        );

        // asserting that TAG2 is untouched, just in case
        assert_eq!(
            rx.get_voting_power(TAG2, &voting_key).unwrap().unwrap(),
            inputs
                .iter()
                .cloned()
                .map(|hir| (hir.voting_group, hir.voting_power))
                .collect::<Vec<_>>()
        );
    }
}
