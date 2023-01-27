use crate::date::Epoch;
use crate::key::BftLeaderId;
use crate::milli::Milli;
use crate::rewards::{Ratio, TaxType};
use crate::value::Value;
use crate::{
    chaintypes::ConsensusType,
    fee::{LinearFee, PerCertificateFee, PerVoteCertificateFee},
    vote::CommitteeId,
};
use chain_addr::Discrimination;
use chain_core::property::WriteError;
use chain_core::{
    packer::Codec,
    property::{DeserializeFromSlice, ReadError, Serialize},
};
use chain_crypto::PublicKey;
use std::{
    fmt::{self, Display, Formatter},
    io::{self, Write},
    num::{NonZeroU32, NonZeroU64},
};
use strum_macros::{AsRefStr, EnumIter, EnumString};
use typed_bytes::ByteBuilder;

/// Possible errors
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Error {
    InvalidTag,
    SizeInvalid,
    BoolInvalid,
    StructureInvalid,
    UnknownString(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::InvalidTag => write!(f, "Invalid config parameter tag"),
            Error::SizeInvalid => write!(f, "Invalid config parameter size"),
            Error::BoolInvalid => write!(f, "Invalid Boolean in config parameter"),
            Error::StructureInvalid => write!(f, "Invalid config parameter structure"),
            Error::UnknownString(s) => write!(f, "Invalid config parameter string '{}'", s),
        }
    }
}

impl std::error::Error for Error {}

impl From<ReadError> for Error {
    fn from(_: ReadError) -> Self {
        Error::StructureInvalid
    }
}

impl From<Error> for ReadError {
    fn from(error: Error) -> ReadError {
        ReadError::StructureInvalid(error.to_string())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigParam {
    Block0Date(Block0Date),
    Discrimination(Discrimination),
    ConsensusVersion(ConsensusType),
    SlotsPerEpoch(u32),
    SlotDuration(u8),
    EpochStabilityDepth(u32),
    ConsensusGenesisPraosActiveSlotsCoeff(Milli),
    BlockContentMaxSize(u32),
    AddBftLeader(BftLeaderId),
    RemoveBftLeader(BftLeaderId),
    LinearFee(LinearFee),
    ProposalExpiration(u32),
    KesUpdateSpeed(u32),
    TreasuryAdd(Value),
    TreasuryParams(TaxType),
    RewardPot(Value),
    RewardParams(RewardParams),
    PerCertificateFees(PerCertificateFee),
    FeesInTreasury(bool),
    RewardLimitNone,
    RewardLimitByAbsoluteStake(Ratio),
    PoolRewardParticipationCapping((NonZeroU32, NonZeroU32)),
    AddCommitteeId(CommitteeId),
    RemoveCommitteeId(CommitteeId),
    PerVoteCertificateFees(PerVoteCertificateFee),
    TransactionMaxExpiryEpochs(u8),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RewardParams {
    Linear {
        constant: u64,
        ratio: Ratio,
        epoch_start: Epoch,
        epoch_rate: NonZeroU32,
    },
    Halving {
        constant: u64,
        ratio: Ratio,
        epoch_start: Epoch,
        epoch_rate: NonZeroU32,
    },
}

// Discriminants can NEVER be 1024 or higher
#[derive(AsRefStr, Clone, Copy, Debug, EnumIter, EnumString, PartialEq, Eq)]
pub enum Tag {
    #[strum(to_string = "discrimination")]
    Discrimination = 1,
    #[strum(to_string = "block0-date")]
    Block0Date = 2,
    #[strum(to_string = "block0-consensus")]
    ConsensusVersion = 3,
    #[strum(to_string = "slots-per-epoch")]
    SlotsPerEpoch = 4,
    #[strum(to_string = "slot-duration")]
    SlotDuration = 5,
    #[strum(to_string = "epoch-stability-depth")]
    EpochStabilityDepth = 6,
    #[strum(to_string = "genesis-praos-param-f")]
    ConsensusGenesisPraosActiveSlotsCoeff = 8,
    #[strum(to_string = "block-content-max-size")]
    BlockContentMaxSize = 9,
    #[strum(to_string = "add-bft-leader")]
    AddBftLeader = 11,
    #[strum(to_string = "remove-bft-leader")]
    RemoveBftLeader = 12,
    #[strum(to_string = "linear-fee")]
    LinearFee = 14,
    #[strum(to_string = "proposal-expiration")]
    ProposalExpiration = 15,
    #[strum(to_string = "kes-update-speed")]
    KesUpdateSpeed = 16,
    #[strum(to_string = "treasury")]
    TreasuryAdd = 17,
    #[strum(to_string = "treasury-params")]
    TreasuryParams = 18,
    #[strum(to_string = "reward-pot")]
    RewardPot = 19,
    #[strum(to_string = "reward-params")]
    RewardParams = 20,
    #[strum(to_string = "per-certificate-fees")]
    PerCertificateFees = 21,
    #[strum(to_string = "fees-in-treasury")]
    FeesInTreasury = 22,
    #[strum(to_string = "reward-limit-none")]
    RewardLimitNone = 23,
    #[strum(to_string = "reward-limit-by-absolute-stake")]
    RewardLimitByAbsoluteStake = 24,
    #[strum(to_string = "pool-reward-participation-capping")]
    PoolRewardParticipationCapping = 25,
    #[strum(to_string = "add-committee-id")]
    AddCommitteeId = 26,
    #[strum(to_string = "remove-committee-id")]
    RemoveCommitteeId = 27,
    #[strum(to_string = "per-vote-certificate-fees")]
    PerVoteCertificateFees = 28,
    #[strum(to_string = "transaction-maximum-expiry-epochs")]
    TransactionMaxExpiryEpochs = 29,
}

impl Tag {
    pub fn from_u16(v: u16) -> Option<Self> {
        match v {
            1 => Some(Tag::Discrimination),
            2 => Some(Tag::Block0Date),
            3 => Some(Tag::ConsensusVersion),
            4 => Some(Tag::SlotsPerEpoch),
            5 => Some(Tag::SlotDuration),
            6 => Some(Tag::EpochStabilityDepth),
            8 => Some(Tag::ConsensusGenesisPraosActiveSlotsCoeff),
            9 => Some(Tag::BlockContentMaxSize),
            11 => Some(Tag::AddBftLeader),
            12 => Some(Tag::RemoveBftLeader),
            14 => Some(Tag::LinearFee),
            15 => Some(Tag::ProposalExpiration),
            16 => Some(Tag::KesUpdateSpeed),
            17 => Some(Tag::TreasuryAdd),
            18 => Some(Tag::TreasuryParams),
            19 => Some(Tag::RewardPot),
            20 => Some(Tag::RewardParams),
            21 => Some(Tag::PerCertificateFees),
            22 => Some(Tag::FeesInTreasury),
            23 => Some(Tag::RewardLimitNone),
            24 => Some(Tag::RewardLimitByAbsoluteStake),
            25 => Some(Tag::PoolRewardParticipationCapping),
            26 => Some(Tag::AddCommitteeId),
            27 => Some(Tag::RemoveCommitteeId),
            28 => Some(Tag::PerVoteCertificateFees),
            29 => Some(Tag::TransactionMaxExpiryEpochs),
            _ => None,
        }
    }
}

impl<'a> From<&'a ConfigParam> for Tag {
    fn from(config_param: &'a ConfigParam) -> Self {
        match config_param {
            ConfigParam::Block0Date(_) => Tag::Block0Date,
            ConfigParam::Discrimination(_) => Tag::Discrimination,
            ConfigParam::ConsensusVersion(_) => Tag::ConsensusVersion,
            ConfigParam::SlotsPerEpoch(_) => Tag::SlotsPerEpoch,
            ConfigParam::SlotDuration(_) => Tag::SlotDuration,
            ConfigParam::EpochStabilityDepth(_) => Tag::EpochStabilityDepth,
            ConfigParam::ConsensusGenesisPraosActiveSlotsCoeff(_) => {
                Tag::ConsensusGenesisPraosActiveSlotsCoeff
            }
            ConfigParam::BlockContentMaxSize(_) => Tag::BlockContentMaxSize,
            ConfigParam::AddBftLeader(_) => Tag::AddBftLeader,
            ConfigParam::RemoveBftLeader(_) => Tag::RemoveBftLeader,
            ConfigParam::LinearFee(_) => Tag::LinearFee,
            ConfigParam::ProposalExpiration(_) => Tag::ProposalExpiration,
            ConfigParam::KesUpdateSpeed(_) => Tag::KesUpdateSpeed,
            ConfigParam::TreasuryAdd(_) => Tag::TreasuryAdd,
            ConfigParam::TreasuryParams(_) => Tag::TreasuryParams,
            ConfigParam::RewardPot(_) => Tag::RewardPot,
            ConfigParam::RewardParams(_) => Tag::RewardParams,
            ConfigParam::PerCertificateFees(_) => Tag::PerCertificateFees,
            ConfigParam::FeesInTreasury(_) => Tag::FeesInTreasury,
            ConfigParam::RewardLimitNone => Tag::RewardLimitNone,
            ConfigParam::RewardLimitByAbsoluteStake(_) => Tag::RewardLimitByAbsoluteStake,
            ConfigParam::PoolRewardParticipationCapping(..) => Tag::PoolRewardParticipationCapping,
            ConfigParam::AddCommitteeId(..) => Tag::AddCommitteeId,
            ConfigParam::RemoveCommitteeId(..) => Tag::RemoveCommitteeId,
            ConfigParam::PerVoteCertificateFees(..) => Tag::PerVoteCertificateFees,
            ConfigParam::TransactionMaxExpiryEpochs(..) => Tag::TransactionMaxExpiryEpochs,
        }
    }
}

impl DeserializeFromSlice for ConfigParam {
    fn deserialize_from_slice(codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        let taglen = TagLen(codec.get_be_u16()?);
        let bytes = codec.get_slice(taglen.get_len())?;
        match taglen.get_tag()? {
            Tag::Block0Date => ConfigParamVariant::from_payload(bytes).map(ConfigParam::Block0Date),
            Tag::Discrimination => {
                ConfigParamVariant::from_payload(bytes).map(ConfigParam::Discrimination)
            }
            Tag::ConsensusVersion => {
                ConfigParamVariant::from_payload(bytes).map(ConfigParam::ConsensusVersion)
            }
            Tag::SlotsPerEpoch => {
                ConfigParamVariant::from_payload(bytes).map(ConfigParam::SlotsPerEpoch)
            }
            Tag::SlotDuration => {
                ConfigParamVariant::from_payload(bytes).map(ConfigParam::SlotDuration)
            }
            Tag::EpochStabilityDepth => {
                ConfigParamVariant::from_payload(bytes).map(ConfigParam::EpochStabilityDepth)
            }
            Tag::ConsensusGenesisPraosActiveSlotsCoeff => ConfigParamVariant::from_payload(bytes)
                .map(ConfigParam::ConsensusGenesisPraosActiveSlotsCoeff),
            Tag::BlockContentMaxSize => {
                ConfigParamVariant::from_payload(bytes).map(ConfigParam::BlockContentMaxSize)
            }
            Tag::AddBftLeader => {
                ConfigParamVariant::from_payload(bytes).map(ConfigParam::AddBftLeader)
            }
            Tag::RemoveBftLeader => {
                ConfigParamVariant::from_payload(bytes).map(ConfigParam::RemoveBftLeader)
            }
            Tag::LinearFee => ConfigParamVariant::from_payload(bytes).map(ConfigParam::LinearFee),
            Tag::ProposalExpiration => {
                ConfigParamVariant::from_payload(bytes).map(ConfigParam::ProposalExpiration)
            }
            Tag::KesUpdateSpeed => {
                ConfigParamVariant::from_payload(bytes).map(ConfigParam::KesUpdateSpeed)
            }
            Tag::TreasuryAdd => {
                ConfigParamVariant::from_payload(bytes).map(ConfigParam::TreasuryAdd)
            }
            Tag::TreasuryParams => {
                ConfigParamVariant::from_payload(bytes).map(ConfigParam::TreasuryParams)
            }
            Tag::RewardPot => ConfigParamVariant::from_payload(bytes).map(ConfigParam::RewardPot),
            Tag::RewardParams => {
                ConfigParamVariant::from_payload(bytes).map(ConfigParam::RewardParams)
            }
            Tag::PerCertificateFees => {
                ConfigParamVariant::from_payload(bytes).map(ConfigParam::PerCertificateFees)
            }
            Tag::FeesInTreasury => {
                ConfigParamVariant::from_payload(bytes).map(ConfigParam::FeesInTreasury)
            }
            Tag::RewardLimitNone => {
                if !bytes.is_empty() {
                    Err(Error::SizeInvalid)
                } else {
                    Ok(ConfigParam::RewardLimitNone)
                }
            }
            Tag::RewardLimitByAbsoluteStake => {
                ConfigParamVariant::from_payload(bytes).map(ConfigParam::RewardLimitByAbsoluteStake)
            }
            Tag::PoolRewardParticipationCapping => ConfigParamVariant::from_payload(bytes)
                .map(ConfigParam::PoolRewardParticipationCapping),
            Tag::AddCommitteeId => {
                ConfigParamVariant::from_payload(bytes).map(ConfigParam::AddCommitteeId)
            }
            Tag::RemoveCommitteeId => {
                ConfigParamVariant::from_payload(bytes).map(ConfigParam::RemoveCommitteeId)
            }
            Tag::PerVoteCertificateFees => {
                ConfigParamVariant::from_payload(bytes).map(ConfigParam::PerVoteCertificateFees)
            }
            Tag::TransactionMaxExpiryEpochs => {
                ConfigParamVariant::from_payload(bytes).map(ConfigParam::TransactionMaxExpiryEpochs)
            }
        }
        .map_err(Into::into)
    }
}

impl Serialize for ConfigParam {
    fn serialized_size(&self) -> usize {
        Codec::u16_size()
            + match self {
                ConfigParam::Block0Date(data) => data.to_payload().len(),
                ConfigParam::Discrimination(data) => data.to_payload().len(),
                ConfigParam::ConsensusVersion(data) => data.to_payload().len(),
                ConfigParam::SlotsPerEpoch(data) => data.to_payload().len(),
                ConfigParam::SlotDuration(data) => data.to_payload().len(),
                ConfigParam::EpochStabilityDepth(data) => data.to_payload().len(),
                ConfigParam::ConsensusGenesisPraosActiveSlotsCoeff(data) => data.to_payload().len(),
                ConfigParam::BlockContentMaxSize(data) => data.to_payload().len(),
                ConfigParam::AddBftLeader(data) => data.to_payload().len(),
                ConfigParam::RemoveBftLeader(data) => data.to_payload().len(),
                ConfigParam::LinearFee(data) => data.to_payload().len(),
                ConfigParam::ProposalExpiration(data) => data.to_payload().len(),
                ConfigParam::KesUpdateSpeed(data) => data.to_payload().as_slice().len(),
                ConfigParam::TreasuryAdd(data) => data.to_payload().len(),
                ConfigParam::TreasuryParams(data) => data.to_payload().len(),
                ConfigParam::RewardPot(data) => data.to_payload().len(),
                ConfigParam::RewardParams(data) => data.to_payload().len(),
                ConfigParam::PerCertificateFees(data) => data.to_payload().len(),
                ConfigParam::FeesInTreasury(data) => data.to_payload().len(),
                ConfigParam::RewardLimitNone => 0,
                ConfigParam::RewardLimitByAbsoluteStake(data) => data.to_payload().len(),
                ConfigParam::PoolRewardParticipationCapping(data) => data.to_payload().len(),
                ConfigParam::AddCommitteeId(data) => data.to_payload().len(),
                ConfigParam::RemoveCommitteeId(data) => data.to_payload().len(),
                ConfigParam::PerVoteCertificateFees(data) => data.to_payload().len(),
                ConfigParam::TransactionMaxExpiryEpochs(data) => data.to_payload().len(),
            }
    }

    fn serialize<W: Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        let tag = Tag::from(self);
        let bytes = match self {
            ConfigParam::Block0Date(data) => data.to_payload(),
            ConfigParam::Discrimination(data) => data.to_payload(),
            ConfigParam::ConsensusVersion(data) => data.to_payload(),
            ConfigParam::SlotsPerEpoch(data) => data.to_payload(),
            ConfigParam::SlotDuration(data) => data.to_payload(),
            ConfigParam::EpochStabilityDepth(data) => data.to_payload(),
            ConfigParam::ConsensusGenesisPraosActiveSlotsCoeff(data) => data.to_payload(),
            ConfigParam::BlockContentMaxSize(data) => data.to_payload(),
            ConfigParam::AddBftLeader(data) => data.to_payload(),
            ConfigParam::RemoveBftLeader(data) => data.to_payload(),
            ConfigParam::LinearFee(data) => data.to_payload(),
            ConfigParam::ProposalExpiration(data) => data.to_payload(),
            ConfigParam::KesUpdateSpeed(data) => data.to_payload(),
            ConfigParam::TreasuryAdd(data) => data.to_payload(),
            ConfigParam::TreasuryParams(data) => data.to_payload(),
            ConfigParam::RewardPot(data) => data.to_payload(),
            ConfigParam::RewardParams(data) => data.to_payload(),
            ConfigParam::PerCertificateFees(data) => data.to_payload(),
            ConfigParam::FeesInTreasury(data) => data.to_payload(),
            ConfigParam::RewardLimitNone => Vec::with_capacity(0),
            ConfigParam::RewardLimitByAbsoluteStake(data) => data.to_payload(),
            ConfigParam::PoolRewardParticipationCapping(data) => data.to_payload(),
            ConfigParam::AddCommitteeId(data) => data.to_payload(),
            ConfigParam::RemoveCommitteeId(data) => data.to_payload(),
            ConfigParam::PerVoteCertificateFees(data) => data.to_payload(),
            ConfigParam::TransactionMaxExpiryEpochs(data) => data.to_payload(),
        };
        let taglen = TagLen::new(tag, bytes.len()).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "initial ent payload too big".to_string(),
            )
        })?;
        codec.put_be_u16(taglen.0)?;
        codec.put_bytes(&bytes)
    }
}

trait ConfigParamVariant: Clone + Eq + PartialEq {
    fn to_payload(&self) -> Vec<u8>;
    fn from_payload(payload: &[u8]) -> Result<Self, Error>;
}

/// Seconds elapsed since 1-Jan-1970 (unix time)
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Block0Date(pub u64);

impl ConfigParamVariant for Block0Date {
    fn to_payload(&self) -> Vec<u8> {
        self.0.to_payload()
    }

    fn from_payload(payload: &[u8]) -> Result<Self, Error> {
        u64::from_payload(payload).map(Block0Date)
    }
}

impl ConfigParamVariant for TaxType {
    fn to_payload(&self) -> Vec<u8> {
        self.serialize_in(ByteBuilder::new()).finalize_as_vec()
    }

    fn from_payload(payload: &[u8]) -> Result<Self, Error> {
        let mut codec = Codec::new(payload);
        let tax_type = TaxType::read_frombuf(&mut codec)?;
        Ok(tax_type)
    }
}

impl ConfigParamVariant for Ratio {
    fn to_payload(&self) -> Vec<u8> {
        let bb: ByteBuilder<Ratio> = ByteBuilder::new();
        bb.u64(self.numerator)
            .u64(self.denominator.get())
            .finalize_as_vec()
    }

    fn from_payload(payload: &[u8]) -> Result<Self, Error> {
        let mut codec = Codec::new(payload);
        let num = codec.get_be_u64()?;
        let denom = codec.get_nz_u64()?;
        Ok(Ratio {
            numerator: num,
            denominator: denom,
        })
    }
}
impl ConfigParamVariant for RewardParams {
    fn to_payload(&self) -> Vec<u8> {
        let bb: ByteBuilder<RewardParams> = match self {
            RewardParams::Linear {
                constant,
                ratio,
                epoch_start,
                epoch_rate,
            } => ByteBuilder::new()
                .u8(1)
                .u64(*constant)
                .u64(ratio.numerator)
                .u64(ratio.denominator.get())
                .u32(*epoch_start)
                .u32(epoch_rate.get()),
            RewardParams::Halving {
                constant,
                ratio,
                epoch_start,
                epoch_rate,
            } => ByteBuilder::new()
                .u8(2)
                .u64(*constant)
                .u64(ratio.numerator)
                .u64(ratio.denominator.get())
                .u32(*epoch_start)
                .u32(epoch_rate.get()),
        };
        bb.finalize_as_vec()
    }

    fn from_payload(payload: &[u8]) -> Result<Self, Error> {
        let mut codec = Codec::new(payload);
        match codec.get_u8()? {
            1 => {
                let start = codec.get_be_u64()?;
                let num = codec.get_be_u64()?;
                let denom = codec.get_nz_u64()?;
                let estart = codec.get_be_u32()?;
                let erate = codec.get_nz_u32()?;
                Ok(RewardParams::Linear {
                    constant: start,
                    ratio: Ratio {
                        numerator: num,
                        denominator: denom,
                    },
                    epoch_start: estart,
                    epoch_rate: erate,
                })
            }
            2 => {
                let start = codec.get_be_u64()?;
                let num = codec.get_be_u64()?;
                let denom = codec.get_nz_u64()?;
                let estart = codec.get_be_u32()?;
                let erate = codec.get_nz_u32()?;
                Ok(RewardParams::Halving {
                    constant: start,
                    ratio: Ratio {
                        numerator: num,
                        denominator: denom,
                    },
                    epoch_start: estart,
                    epoch_rate: erate,
                })
            }
            _ => Err(Error::InvalidTag),
        }
    }
}

impl ConfigParamVariant for Value {
    fn to_payload(&self) -> Vec<u8> {
        self.0.to_payload()
    }

    fn from_payload(payload: &[u8]) -> Result<Self, Error> {
        u64::from_payload(payload).map(Value)
    }
}

const VAL_PROD: u8 = 1;
const VAL_TEST: u8 = 2;

impl ConfigParamVariant for Discrimination {
    fn to_payload(&self) -> Vec<u8> {
        match self {
            Discrimination::Production => vec![VAL_PROD],
            Discrimination::Test => vec![VAL_TEST],
        }
    }

    fn from_payload(payload: &[u8]) -> Result<Self, Error> {
        if payload.len() != 1 {
            return Err(Error::SizeInvalid);
        };
        match payload[0] {
            VAL_PROD => Ok(Discrimination::Production),
            VAL_TEST => Ok(Discrimination::Test),
            _ => Err(Error::StructureInvalid),
        }
    }
}

impl ConfigParamVariant for ConsensusType {
    fn to_payload(&self) -> Vec<u8> {
        (*self as u16).to_be_bytes().to_vec()
    }

    fn from_payload(payload: &[u8]) -> Result<Self, Error> {
        let mut bytes = 0u16.to_ne_bytes();
        if payload.len() != bytes.len() {
            return Err(Error::SizeInvalid);
        };
        bytes.copy_from_slice(payload);
        let integer = u16::from_be_bytes(bytes);
        ConsensusType::from_u16(integer).ok_or(Error::StructureInvalid)
    }
}

impl ConfigParamVariant for BftLeaderId {
    fn to_payload(&self) -> Vec<u8> {
        self.as_ref().to_vec()
    }

    fn from_payload(payload: &[u8]) -> Result<Self, Error> {
        PublicKey::from_binary(payload)
            .map(Into::into)
            .map_err(|_| Error::SizeInvalid)
    }
}

impl ConfigParamVariant for bool {
    fn to_payload(&self) -> Vec<u8> {
        vec![u8::from(*self)]
    }

    fn from_payload(payload: &[u8]) -> Result<Self, Error> {
        match payload.len() {
            1 => match payload[0] {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(Error::BoolInvalid),
            },
            _ => Err(Error::SizeInvalid),
        }
    }
}

impl ConfigParamVariant for u8 {
    fn to_payload(&self) -> Vec<u8> {
        vec![*self]
    }

    fn from_payload(payload: &[u8]) -> Result<Self, Error> {
        match payload.len() {
            1 => Ok(payload[0]),
            _ => Err(Error::SizeInvalid),
        }
    }
}

impl ConfigParamVariant for u64 {
    fn to_payload(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn from_payload(payload: &[u8]) -> Result<Self, Error> {
        let mut bytes = Self::default().to_ne_bytes();
        if payload.len() != bytes.len() {
            return Err(Error::SizeInvalid);
        };
        bytes.copy_from_slice(payload);
        Ok(Self::from_be_bytes(bytes))
    }
}

impl ConfigParamVariant for u32 {
    fn to_payload(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn from_payload(payload: &[u8]) -> Result<Self, Error> {
        let mut bytes = Self::default().to_ne_bytes();
        if payload.len() != bytes.len() {
            return Err(Error::SizeInvalid);
        };
        bytes.copy_from_slice(payload);
        Ok(Self::from_be_bytes(bytes))
    }
}

impl ConfigParamVariant for (NonZeroU32, NonZeroU32) {
    fn to_payload(&self) -> Vec<u8> {
        let bb: ByteBuilder<()> = ByteBuilder::new();
        bb.u32(self.0.get()).u32(self.1.get()).finalize_as_vec()
    }

    fn from_payload(payload: &[u8]) -> Result<Self, Error> {
        let mut codec = Codec::new(payload);
        let x = codec.get_nz_u32()?;
        let y = codec.get_nz_u32()?;
        Ok((x, y))
    }
}

impl ConfigParamVariant for Milli {
    fn to_payload(&self) -> Vec<u8> {
        self.to_millis().to_payload()
    }

    fn from_payload(payload: &[u8]) -> Result<Self, Error> {
        u64::from_payload(payload).map(Milli::from_millis)
    }
}

impl ConfigParamVariant for LinearFee {
    fn to_payload(&self) -> Vec<u8> {
        let mut v = self.constant.to_payload();
        v.extend(self.coefficient.to_payload());
        v.extend(self.certificate.to_payload());
        v
    }

    fn from_payload(payload: &[u8]) -> Result<Self, Error> {
        if payload.len() != 3 * 8 {
            return Err(Error::SizeInvalid);
        }
        Ok(LinearFee {
            constant: u64::from_payload(&payload[0..8])?,
            coefficient: u64::from_payload(&payload[8..16])?,
            certificate: u64::from_payload(&payload[16..24])?,
            per_certificate_fees: PerCertificateFee::default(),
            per_vote_certificate_fees: PerVoteCertificateFee::default(),
        })
    }
}

impl ConfigParamVariant for PerCertificateFee {
    fn to_payload(&self) -> Vec<u8> {
        let mut v = self
            .certificate_pool_registration
            .map(|v| v.get())
            .unwrap_or(0)
            .to_payload();
        v.extend(
            self.certificate_stake_delegation
                .map(|v| v.get())
                .unwrap_or(0)
                .to_payload(),
        );
        v.extend(
            self.certificate_owner_stake_delegation
                .map(|v| v.get())
                .unwrap_or(0)
                .to_payload(),
        );
        v
    }

    fn from_payload(payload: &[u8]) -> Result<Self, Error> {
        if payload.len() != 3 * 8 {
            return Err(Error::SizeInvalid);
        }
        Ok(PerCertificateFee {
            certificate_pool_registration: NonZeroU64::new(u64::from_payload(&payload[0..8])?),
            certificate_stake_delegation: NonZeroU64::new(u64::from_payload(&payload[8..16])?),
            certificate_owner_stake_delegation: NonZeroU64::new(u64::from_payload(
                &payload[16..24],
            )?),
        })
    }
}

impl ConfigParamVariant for PerVoteCertificateFee {
    fn to_payload(&self) -> Vec<u8> {
        let mut v = self
            .certificate_vote_plan
            .map(|v| v.get())
            .unwrap_or(0)
            .to_payload();
        v.extend(
            self.certificate_vote_cast
                .map(|v| v.get())
                .unwrap_or(0)
                .to_payload(),
        );
        v
    }

    fn from_payload(payload: &[u8]) -> Result<Self, Error> {
        if payload.len() != 2 * 8 {
            return Err(Error::SizeInvalid);
        }
        Ok(Self {
            certificate_vote_plan: NonZeroU64::new(u64::from_payload(&payload[0..8])?),
            certificate_vote_cast: NonZeroU64::new(u64::from_payload(&payload[8..16])?),
        })
    }
}

impl ConfigParamVariant for CommitteeId {
    fn to_payload(&self) -> Vec<u8> {
        self.as_ref().to_vec()
    }

    fn from_payload(payload: &[u8]) -> Result<Self, Error> {
        Self::try_from(payload).map_err(|err| Error::UnknownString(err.to_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TagLen(u16);

const MAXIMUM_LEN: usize = 512;

impl TagLen {
    pub fn new(tag: Tag, len: usize) -> Option<Self> {
        if len < MAXIMUM_LEN {
            Some(TagLen((tag as u16) << 9 | len as u16))
        } else {
            None
        }
    }

    pub fn get_len(self) -> usize {
        (self.0 & 0b1_1111_1111) as usize
    }

    pub fn get_tag(self) -> Result<Tag, Error> {
        Tag::from_u16(self.0 >> 9).ok_or(Error::InvalidTag)
    }
}

#[cfg(any(test, feature = "property-test-api"))]
mod test {
    use super::*;
    #[cfg(test)]
    use quickcheck::TestResult;
    use quickcheck::{Arbitrary, Gen};
    use strum::IntoEnumIterator;

    quickcheck! {
        fn tag_len_computation_correct(tag: Tag, len: usize) -> TestResult {
            let len = len % MAXIMUM_LEN;
            let tag_len = TagLen::new(tag, len).unwrap();

            assert_eq!(Ok(tag), tag_len.get_tag(), "Invalid tag");
            assert_eq!(len, tag_len.get_len(), "Invalid len");
            TestResult::passed()
        }

        fn linear_fee_to_payload_from_payload(fee: LinearFee) -> TestResult {
            let payload = fee.to_payload();
            let decoded = LinearFee::from_payload(&payload).unwrap();

            TestResult::from_bool(fee == decoded)
        }

        fn per_certificate_fee_to_payload_from_payload(fee: PerCertificateFee) -> TestResult {
            let payload = fee.to_payload();
            let decoded = PerCertificateFee::from_payload(&payload).unwrap();

            TestResult::from_bool(fee == decoded)
        }

        fn config_param_serialize_readable(param: ConfigParam) -> bool {
            use chain_core::property::Serialize as _;
            let bytes = param.serialize_as_vec().unwrap();
            let decoded = ConfigParam::deserialize_from_slice(&mut Codec::new(bytes.as_slice())).unwrap();

            param == decoded
        }
    }

    impl Arbitrary for Tag {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let idx = usize::arbitrary(g) % Tag::iter().count();
            Tag::iter().nth(idx).unwrap()
        }
    }

    impl Arbitrary for Block0Date {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Block0Date(Arbitrary::arbitrary(g))
        }
    }

    impl Arbitrary for Ratio {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Ratio {
                numerator: Arbitrary::arbitrary(g),
                denominator: NonZeroU64::new(Arbitrary::arbitrary(g))
                    .unwrap_or_else(|| NonZeroU64::new(1).unwrap()),
            }
        }
    }

    impl Arbitrary for RewardParams {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            if bool::arbitrary(g) {
                RewardParams::Linear {
                    constant: Arbitrary::arbitrary(g),
                    ratio: Arbitrary::arbitrary(g),
                    epoch_start: Arbitrary::arbitrary(g),
                    epoch_rate: NonZeroU32::new(20).unwrap(),
                }
            } else {
                RewardParams::Halving {
                    constant: Arbitrary::arbitrary(g),
                    ratio: Arbitrary::arbitrary(g),
                    epoch_start: Arbitrary::arbitrary(g),
                    epoch_rate: NonZeroU32::new(20).unwrap(),
                }
            }
        }
    }

    impl Arbitrary for ConfigParam {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            match u8::arbitrary(g) % 30 {
                0 => ConfigParam::Block0Date(Arbitrary::arbitrary(g)),
                1 => ConfigParam::Discrimination(Arbitrary::arbitrary(g)),
                2 => ConfigParam::ConsensusVersion(Arbitrary::arbitrary(g)),
                3 => ConfigParam::SlotsPerEpoch(Arbitrary::arbitrary(g)),
                4 => {
                    let mut slot_duration = u8::arbitrary(g);
                    if slot_duration == 0 {
                        slot_duration += 1;
                    }
                    ConfigParam::SlotDuration(slot_duration)
                }
                5 => ConfigParam::ConsensusGenesisPraosActiveSlotsCoeff(Arbitrary::arbitrary(g)),
                6 => ConfigParam::BlockContentMaxSize(Arbitrary::arbitrary(g)),
                7 => ConfigParam::AddBftLeader(Arbitrary::arbitrary(g)),
                8 => ConfigParam::RemoveBftLeader(Arbitrary::arbitrary(g)),
                9 => ConfigParam::LinearFee(Arbitrary::arbitrary(g)),
                10 => ConfigParam::ProposalExpiration(Arbitrary::arbitrary(g)),
                11 => ConfigParam::TreasuryAdd(Arbitrary::arbitrary(g)),
                12 => ConfigParam::RewardPot(Arbitrary::arbitrary(g)),
                13 => ConfigParam::RewardParams(Arbitrary::arbitrary(g)),
                14 => ConfigParam::PerCertificateFees(Arbitrary::arbitrary(g)),
                15 => ConfigParam::FeesInTreasury(Arbitrary::arbitrary(g)),
                16 => ConfigParam::AddCommitteeId(Arbitrary::arbitrary(g)),
                17 => ConfigParam::RemoveCommitteeId(Arbitrary::arbitrary(g)),
                18 => ConfigParam::PerVoteCertificateFees(Arbitrary::arbitrary(g)),
                19 => ConfigParam::RewardPot(Arbitrary::arbitrary(g)),
                20 => ConfigParam::RewardParams(Arbitrary::arbitrary(g)),
                21 => ConfigParam::RewardParams(Arbitrary::arbitrary(g)),
                22 => ConfigParam::FeesInTreasury(Arbitrary::arbitrary(g)),
                23 => ConfigParam::RewardLimitNone,
                24 => ConfigParam::RewardLimitByAbsoluteStake(Arbitrary::arbitrary(g)),
                25 => ConfigParam::PoolRewardParticipationCapping(Arbitrary::arbitrary(g)),
                26 => ConfigParam::AddCommitteeId(Arbitrary::arbitrary(g)),
                27 => ConfigParam::RemoveCommitteeId(Arbitrary::arbitrary(g)),
                28 => ConfigParam::PerCertificateFees(Arbitrary::arbitrary(g)),
                29 => ConfigParam::TransactionMaxExpiryEpochs(Arbitrary::arbitrary(g)),
                _ => unreachable!(),
            }
        }
    }
}
