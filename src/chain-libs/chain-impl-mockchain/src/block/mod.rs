//! Representation of the block in the mockchain.
use crate::fragment::Fragment;
use chain_core::{
    packer::Codec,
    property::{self, Deserialize, ReadError, Serialize, WriteError},
};

use std::slice;

mod builder;
mod header;

#[cfg(any(test, feature = "property-test-api"))]
pub mod test;

//pub use self::builder::BlockBuilder;
pub use crate::fragment::{BlockContentHash, BlockContentSize, Contents, ContentsBuilder};

pub use crate::header::{
    BftProof, BftSignature, Common, GenesisPraosProof, Header, HeaderId, KesSignature, Proof,
};

pub use builder::builder;

pub use crate::header::{BlockVersion, ChainLength};

pub use crate::date::{BlockDate, BlockDateParseError, Epoch, SlotId};

/// `Block` is an element of the blockchain it contains multiple
/// transaction and a reference to the parent block. Alongside
/// with the position of that block in the chain.
#[derive(Debug, Clone)]
pub struct Block {
    header: Header,
    contents: Contents,
}

impl PartialEq for Block {
    fn eq(&self, rhs: &Self) -> bool {
        self.header.hash() == rhs.header.hash()
    }
}
impl Eq for Block {}

impl Block {
    /// Does not validate that the block is consistent
    pub(super) fn new_unchecked(header: Header, contents: Contents) -> Self {
        Self { header, contents }
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn contents(&self) -> &Contents {
        &self.contents
    }

    pub fn fragments(&self) -> impl Iterator<Item = &Fragment> {
        self.contents.iter()
    }
}

impl property::Block for Block {
    type Id = HeaderId;
    type Date = BlockDate;
    type Version = BlockVersion;
    type ChainLength = ChainLength;

    /// Identifier of the block, currently the hash of the
    /// serialized transaction.
    fn id(&self) -> Self::Id {
        self.header.hash()
    }

    /// Id of the parent block.
    fn parent_id(&self) -> Self::Id {
        self.header.block_parent_hash()
    }

    /// Date of the block.
    fn date(&self) -> Self::Date {
        self.header.block_date()
    }

    fn version(&self) -> Self::Version {
        self.header.block_version()
    }

    fn chain_length(&self) -> Self::ChainLength {
        self.header.chain_length()
    }
}

impl Serialize for Block {
    fn serialized_size(&self) -> usize {
        let mut res = self.header.serialized_size();
        for message in self.contents.iter() {
            res += message.serialized_size();
        }
        res
    }

    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        self.header.serialize(codec)?;
        for message in self.contents.iter() {
            message.serialize(codec)?;
        }
        Ok(())
    }
}

impl Deserialize for Block {
    fn deserialize<R: std::io::Read>(codec: &mut Codec<R>) -> Result<Self, ReadError> {
        let header = Header::deserialize(codec)?;
        let mut remaining_content_size = header.block_content_size() as usize;
        let mut contents = ContentsBuilder::new();

        while remaining_content_size > 0 {
            let message = Fragment::deserialize(codec)?;
            let message_size = message.serialized_size();

            if message_size > remaining_content_size {
                return Err(ReadError::StructureInvalid(format!(
                    "{} bytes remaining according to the header but got a fragment of size {}",
                    message_size, remaining_content_size
                )));
            }

            contents.push(message);

            remaining_content_size -= message_size;
        }

        let contents: Contents = contents.into();
        let (content_hash, _content_size) = contents.compute_hash_size();

        if header.block_content_hash() != content_hash {
            return Err(ReadError::InvalidData(format!(
                "Inconsistent block content hash in header: block {} header {}",
                content_hash,
                header.block_content_hash()
            )));
        }

        Ok(Block { header, contents })
    }
}

impl<'a> property::HasFragments<'a> for &'a Block {
    type Fragment = Fragment;
    type Fragments = slice::Iter<'a, Fragment>;
    fn fragments(self) -> Self::Fragments {
        self.contents.iter_slice()
    }
}

impl property::HasHeader for Block {
    type Header = Header;
    fn header(&self) -> Self::Header {
        self.header.clone()
    }
}
