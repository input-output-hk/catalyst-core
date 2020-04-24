//! Representation of the block in the mockchain.
use crate::fragment::{Fragment, FragmentRaw};
use chain_core::mempack::{read_from_raw, ReadBuf, ReadError, Readable};
use chain_core::property;

use std::slice;

mod builder;
mod header;
mod headerraw;

#[cfg(any(test, feature = "property-test-api"))]
pub mod test;

//pub use self::builder::BlockBuilder;
pub use crate::fragment::{BlockContentHash, BlockContentSize, Contents, ContentsBuilder};

pub use self::headerraw::HeaderRaw;
pub use crate::header::{
    BftProof, BftSignature, Common, GenesisPraosProof, Header, HeaderId, KESSignature, Proof,
};

pub use builder::builder;

pub use crate::header::{BlockVersion, ChainLength};

pub use crate::date::{BlockDate, BlockDateParseError, Epoch, SlotId};

/// `Block` is an element of the blockchain it contains multiple
/// transaction and a reference to the parent block. Alongside
/// with the position of that block in the chain.
#[derive(Debug, Clone)]
pub struct Block {
    pub header: Header,
    pub contents: Contents,
}

impl PartialEq for Block {
    fn eq(&self, rhs: &Self) -> bool {
        self.header.hash() == rhs.header.hash()
    }
}
impl Eq for Block {}

impl Block {
    pub fn is_consistent(&self) -> bool {
        let (content_hash, content_size) = self.contents.compute_hash_size();

        content_hash == self.header.block_content_hash()
            && content_size == self.header.block_content_size()
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

impl property::Serialize for Block {
    type Error = std::io::Error;

    fn serialize<W: std::io::Write>(&self, mut writer: W) -> Result<(), Self::Error> {
        let header_raw = {
            let mut v = Vec::new();
            self.header.serialize(&mut v)?;
            HeaderRaw(v)
        };
        header_raw.serialize(&mut writer)?;

        for message in self.contents.iter() {
            let message_raw = message.to_raw();
            message_raw.serialize(&mut writer)?;
        }
        Ok(())
    }
}

impl property::Deserialize for Block {
    type Error = std::io::Error;

    fn deserialize<R: std::io::BufRead>(mut reader: R) -> Result<Self, Self::Error> {
        let header_raw = HeaderRaw::deserialize(&mut reader)?;
        let header = read_from_raw::<Header>(header_raw.as_ref())?;

        let mut serialized_content_size = header.block_content_size();
        let mut contents = ContentsBuilder::new();

        while serialized_content_size > 0 {
            let message_raw = FragmentRaw::deserialize(&mut reader)?;
            let message_size = message_raw.size_bytes_plus_size();

            // return error here if message serialize sized is bigger than remaining size

            let message = Fragment::from_raw(&message_raw)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
            contents.push(message);

            serialized_content_size -= message_size as u32;
        }

        Ok(Block {
            header,
            contents: contents.into(),
        })
    }
}

impl Readable for Block {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        let header_size = buf.get_u16()? as usize;
        let mut header_buf = buf.split_to(header_size)?;
        let header = Header::read(&mut header_buf)?;

        let mut remaining_content_size = header.block_content_size();
        let mut contents = ContentsBuilder::new();

        while remaining_content_size > 0 {
            let message_size = buf.get_u16()?;
            let mut message_buf = buf.split_to(message_size as usize)?;

            // return error here if message serialize sized is bigger than remaining size

            let message = Fragment::read(&mut message_buf)?;
            contents.push(message);

            remaining_content_size -= 2 + message_size as u32;
        }

        Ok(Block {
            header,
            contents: contents.into(),
        })
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
