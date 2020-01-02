use crate::date::BlockDate;
use crate::header::{BlockVersion, ChainLength, Header, HeaderId};
use chain_core::property;

impl property::ChainLength for ChainLength {
    fn next(&self) -> Self {
        self.increase()
    }
}

impl property::Header for Header {
    type Id = HeaderId;
    type Date = BlockDate;
    type Version = BlockVersion;
    type ChainLength = ChainLength;

    fn id(&self) -> Self::Id {
        self.hash()
    }
    fn parent_id(&self) -> Self::Id {
        self.block_parent_hash()
    }
    fn chain_length(&self) -> Self::ChainLength {
        self.chain_length()
    }
    fn date(&self) -> Self::Date {
        self.block_date()
    }
    fn version(&self) -> Self::Version {
        self.block_version()
    }
}
