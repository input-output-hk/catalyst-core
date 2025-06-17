use super::cstruct;
use crate::chaintypes::ConsensusType;
use std::num::NonZeroUsize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnyBlockVersion {
    Supported(BlockVersion),
    Unsupported(u8),
}

impl AnyBlockVersion {
    pub fn try_into_block_version(self) -> Option<BlockVersion> {
        match self {
            AnyBlockVersion::Supported(version) => Some(version),
            AnyBlockVersion::Unsupported(_) => None,
        }
    }
}

impl PartialEq<BlockVersion> for AnyBlockVersion {
    fn eq(&self, other: &BlockVersion) -> bool {
        match self {
            AnyBlockVersion::Supported(version) => version == other,
            AnyBlockVersion::Unsupported(_) => false,
        }
    }
}

impl From<u8> for AnyBlockVersion {
    fn from(n: u8) -> Self {
        match BlockVersion::from_u8(n) {
            Some(supported) => AnyBlockVersion::Supported(supported),
            None => AnyBlockVersion::Unsupported(n),
        }
    }
}

impl From<AnyBlockVersion> for u8 {
    fn from(block_version: AnyBlockVersion) -> u8 {
        match block_version {
            AnyBlockVersion::Supported(version) => version as u8,
            AnyBlockVersion::Unsupported(n) => n,
        }
    }
}

impl From<BlockVersion> for AnyBlockVersion {
    fn from(version: BlockVersion) -> Self {
        AnyBlockVersion::Supported(version)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockVersion {
    Genesis,
    Ed25519Signed,
    KesVrfproof,
}

impl BlockVersion {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            cstruct::VERSION_UNSIGNED => Some(BlockVersion::Genesis),
            cstruct::VERSION_BFT => Some(BlockVersion::Ed25519Signed),
            cstruct::VERSION_GP => Some(BlockVersion::KesVrfproof),
            _ => None,
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            BlockVersion::Genesis => cstruct::VERSION_UNSIGNED,
            BlockVersion::Ed25519Signed => cstruct::VERSION_BFT,
            BlockVersion::KesVrfproof => cstruct::VERSION_GP,
        }
    }

    pub const fn get_size(self) -> NonZeroUsize {
        const SIZE: [NonZeroUsize; 3] = [
            NonZeroUsize::new(cstruct::HEADER_COMMON_SIZE).unwrap(),
            NonZeroUsize::new(cstruct::HEADER_BFT_SIZE).unwrap(),
            NonZeroUsize::new(cstruct::HEADER_GP_SIZE).unwrap(),
        ];
        SIZE[self as usize]
    }

    pub const fn get_auth_size(self) -> NonZeroUsize {
        const SIZE: [NonZeroUsize; 3] = [
            NonZeroUsize::new(cstruct::HEADER_COMMON_SIZE).unwrap(),
            NonZeroUsize::new(cstruct::HEADER_BFT_AUTHED_SIZE).unwrap(),
            NonZeroUsize::new(cstruct::HEADER_GP_AUTHED_SIZE).unwrap(),
        ];
        SIZE[self as usize]
    }

    pub fn to_consensus_type(self) -> Option<ConsensusType> {
        match self {
            BlockVersion::Genesis => None,
            BlockVersion::Ed25519Signed => Some(ConsensusType::Bft),
            BlockVersion::KesVrfproof => Some(ConsensusType::GenesisPraos),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::chaintypes::ConsensusType;
    use crate::header::{AnyBlockVersion, BlockVersion};
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    #[test]
    pub fn try_into_block_version() {
        assert_eq!(
            AnyBlockVersion::Supported(BlockVersion::Genesis).try_into_block_version(),
            Some(BlockVersion::Genesis)
        );
        assert_eq!(
            AnyBlockVersion::Supported(BlockVersion::Ed25519Signed).try_into_block_version(),
            Some(BlockVersion::Ed25519Signed)
        );
        assert_eq!(
            AnyBlockVersion::Supported(BlockVersion::KesVrfproof).try_into_block_version(),
            Some(BlockVersion::KesVrfproof)
        );
        assert_eq!(
            AnyBlockVersion::Unsupported(0).try_into_block_version(),
            None
        );
    }

    #[test]
    pub fn equality() {
        assert_eq!(
            AnyBlockVersion::Supported(BlockVersion::Genesis),
            BlockVersion::Genesis
        );
        assert_eq!(
            AnyBlockVersion::Supported(BlockVersion::Ed25519Signed),
            BlockVersion::Ed25519Signed
        );
        assert_eq!(
            AnyBlockVersion::Supported(BlockVersion::KesVrfproof),
            BlockVersion::KesVrfproof
        );
        assert!(AnyBlockVersion::Unsupported(0) != BlockVersion::KesVrfproof);
        assert!(AnyBlockVersion::Unsupported(0) != BlockVersion::Ed25519Signed);
        assert!(AnyBlockVersion::Unsupported(0) != BlockVersion::KesVrfproof);
    }

    #[quickcheck]
    pub fn conversion_u8(block_version: AnyBlockVersion) -> TestResult {
        let bytes: u8 = block_version.into();
        let new_block_version: AnyBlockVersion = AnyBlockVersion::from(bytes);
        TestResult::from_bool(block_version == new_block_version)
    }

    #[quickcheck]
    pub fn from_block_version(block_version: BlockVersion) -> TestResult {
        let right_version = AnyBlockVersion::Supported(block_version);
        let left_version: AnyBlockVersion = block_version.into();
        TestResult::from_bool(left_version == right_version)
    }

    #[test]
    pub fn to_consensus_type() {
        assert_eq!(BlockVersion::Genesis.to_consensus_type(), None);
        assert_eq!(
            BlockVersion::Ed25519Signed.to_consensus_type(),
            Some(ConsensusType::Bft)
        );
        assert_eq!(
            BlockVersion::KesVrfproof.to_consensus_type(),
            Some(ConsensusType::GenesisPraos)
        );
    }
}
