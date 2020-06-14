//! define the Blockchain settings
//!

use crate::fragment::{config::ConfigParams, BlockContentSize};
use crate::milli::Milli;
use crate::update;
use crate::{
    chaineval::PraosNonce,
    chaintypes::ConsensusType,
    config::{ConfigParam, RewardParams},
    fee::LinearFee,
    key::BftLeaderId,
    rewards,
    vote::CommitteeId,
};
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::num::NonZeroU32;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Settings {
    pub consensus_version: ConsensusType,
    pub consensus_nonce: PraosNonce,
    pub slots_per_epoch: u32,
    pub slot_duration: u8,
    pub epoch_stability_depth: u32,
    pub active_slots_coeff: ActiveSlotsCoeff,
    pub block_content_max_size: BlockContentSize,
    pub bft_leaders: Arc<Vec<BftLeaderId>>,
    pub linear_fees: Arc<LinearFee>,
    /// The number of epochs that a proposal remains valid. To be
    /// precise, if a proposal is made at date (epoch_p, slot), then
    /// it expires at the start of epoch 'epoch_p +
    /// proposal_expiration + 1'. FIXME: make updateable.
    pub proposal_expiration: u32,
    pub reward_params: Option<RewardParams>,
    pub treasury_params: Option<rewards::TaxType>,
    pub fees_goes_to: FeesGoesTo,
    pub rewards_limit: rewards::Limit,
    pub pool_participation_capping: Option<(NonZeroU32, NonZeroU32)>,

    pub committees: Arc<Vec<CommitteeId>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActiveSlotsCoeffError {
    InvalidValue(Milli),
}

impl fmt::Display for ActiveSlotsCoeffError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ActiveSlotsCoeffError::InvalidValue(v) => {
                write!(f, "Invalid value {}, should be in range (0,1]", v)
            }
        }
    }
}

impl Error for ActiveSlotsCoeffError {}

/// Active slots coefficient used for calculating minimum stake to become slot leader candidate
/// Described in Ouroboros Praos paper, also referred to as parameter F of phi function
/// Always in range (0, 1]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActiveSlotsCoeff(Milli);

impl TryFrom<Milli> for ActiveSlotsCoeff {
    type Error = ActiveSlotsCoeffError;

    fn try_from(value: Milli) -> Result<Self, Self::Error> {
        if value > Milli::ZERO && value <= Milli::ONE {
            Ok(ActiveSlotsCoeff(value))
        } else {
            Err(ActiveSlotsCoeffError::InvalidValue(value))
        }
    }
}

impl From<ActiveSlotsCoeff> for Milli {
    fn from(coeff: ActiveSlotsCoeff) -> Milli {
        coeff.0
    }
}

impl From<ActiveSlotsCoeff> for f64 {
    fn from(coeff: ActiveSlotsCoeff) -> f64 {
        coeff.0.to_float()
    }
}

/// Fees nSettings
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeesGoesTo {
    /// Move the fees to the rewards; this is the common mode of blockchain operation.
    Rewards,
    /// Move the fees directly to treasury. this is not a recommended settings, as
    /// it fundamentally change the dynamic of the blockchain operation.
    Treasury,
}

impl Default for FeesGoesTo {
    fn default() -> Self {
        FeesGoesTo::Rewards
    }
}

pub const SLOTS_PERCENTAGE_RANGE: u8 = 100;

impl Settings {
    pub fn new() -> Self {
        Self {
            consensus_version: ConsensusType::Bft,
            consensus_nonce: PraosNonce::zero(),
            slots_per_epoch: 1,
            slot_duration: 10,         // 10 sec
            epoch_stability_depth: 10, // num of block
            active_slots_coeff: ActiveSlotsCoeff::try_from(Milli::HALF).unwrap(),
            block_content_max_size: 102_400,
            bft_leaders: Arc::new(Vec::new()),
            linear_fees: Arc::new(LinearFee::new(0, 0, 0)),
            proposal_expiration: 100,
            reward_params: None,
            treasury_params: None,
            fees_goes_to: FeesGoesTo::Rewards,
            rewards_limit: rewards::Limit::None,
            pool_participation_capping: None,
            committees: Arc::new(Vec::new()),
        }
    }

    pub fn linear_fees(&self) -> LinearFee {
        *self.linear_fees
    }

    pub fn apply(&self, changes: &ConfigParams) -> Result<Self, update::Error> {
        let mut new_state = self.clone();
        let mut per_certificate_fees = None;
        let mut per_vote_certificate_fees = None;

        for param in changes.iter() {
            match param {
                ConfigParam::Block0Date(_)
                | ConfigParam::Discrimination(_)
                | ConfigParam::TreasuryAdd(_)
                | ConfigParam::RewardPot(_)
                | ConfigParam::KESUpdateSpeed(_) => {
                    return Err(update::Error::ReadOnlySetting);
                }
                ConfigParam::ConsensusVersion(d) => {
                    new_state.consensus_version = *d;
                }
                ConfigParam::SlotsPerEpoch(d) => {
                    new_state.slots_per_epoch = *d;
                }
                ConfigParam::SlotDuration(d) => {
                    new_state.slot_duration = *d;
                }
                ConfigParam::EpochStabilityDepth(d) => {
                    new_state.epoch_stability_depth = *d;
                }
                ConfigParam::ConsensusGenesisPraosActiveSlotsCoeff(d) => {
                    new_state.active_slots_coeff = ActiveSlotsCoeff::try_from(*d)?;
                }
                ConfigParam::BlockContentMaxSize(d) => {
                    new_state.block_content_max_size = *d;
                }
                ConfigParam::AddBftLeader(d) => {
                    // FIXME: O(n)
                    let mut v = new_state.bft_leaders.to_vec();
                    v.push(d.clone());
                    new_state.bft_leaders = Arc::new(v);

                    // BFT Leader are automatically promoted committee too
                    // FIXME: O(n)
                    let mut v = new_state.committees.to_vec();
                    v.push(d.as_public_key().clone().into());
                    new_state.committees = Arc::new(v);
                }
                ConfigParam::RemoveBftLeader(d) => {
                    new_state.bft_leaders = Arc::new(
                        new_state
                            .bft_leaders
                            .iter()
                            .filter(|leader| *leader != d)
                            .cloned()
                            .collect(),
                    );
                }
                ConfigParam::LinearFee(d) => {
                    new_state.linear_fees = Arc::new(*d);
                }
                ConfigParam::ProposalExpiration(d) => {
                    new_state.proposal_expiration = *d;
                }
                ConfigParam::RewardParams(rp) => {
                    new_state.reward_params = Some(rp.clone());
                }
                ConfigParam::TreasuryParams(rp) => {
                    new_state.treasury_params = Some(*rp);
                }
                ConfigParam::PerCertificateFees(pcf) => {
                    per_certificate_fees = Some(pcf);
                }
                ConfigParam::PerVoteCertificateFees(pcf) => {
                    per_vote_certificate_fees = Some(pcf);
                }
                ConfigParam::FeesInTreasury(value) => {
                    new_state.fees_goes_to = if *value {
                        FeesGoesTo::Treasury
                    } else {
                        FeesGoesTo::Rewards
                    };
                }
                ConfigParam::RewardLimitNone => new_state.rewards_limit = rewards::Limit::None,
                ConfigParam::RewardLimitByAbsoluteStake(ratio) => {
                    new_state.rewards_limit = rewards::Limit::ByStakeAbsolute(*ratio)
                }
                ConfigParam::PoolRewardParticipationCapping(r) => {
                    new_state.pool_participation_capping = Some(*r)
                }
                ConfigParam::AddCommitteeId(committee_id) => {
                    // FIXME: O(n)
                    let mut v = new_state.committees.to_vec();
                    v.push(*committee_id);
                    new_state.committees = Arc::new(v);
                }
                ConfigParam::RemoveCommitteeId(committee_id) => {
                    new_state.committees = Arc::new(
                        new_state
                            .committees
                            .iter()
                            .filter(|cid| *cid != committee_id)
                            .cloned()
                            .collect(),
                    );
                }
            }
        }

        if let Some(pcf) = per_certificate_fees {
            Arc::get_mut(&mut new_state.linear_fees)
                .unwrap()
                .per_certificate_fees(*pcf);
        }

        if let Some(pcf) = per_vote_certificate_fees {
            Arc::get_mut(&mut new_state.linear_fees)
                .unwrap()
                .per_vote_certificate_fees(*pcf);
        }

        Ok(new_state)
    }

    pub fn to_config_params(&self) -> ConfigParams {
        let mut params = ConfigParams::new();

        params.push(ConfigParam::ConsensusVersion(self.consensus_version));
        params.push(ConfigParam::SlotsPerEpoch(self.slots_per_epoch));
        params.push(ConfigParam::SlotDuration(self.slot_duration));
        params.push(ConfigParam::EpochStabilityDepth(self.epoch_stability_depth));
        params.push(ConfigParam::ConsensusGenesisPraosActiveSlotsCoeff(
            self.active_slots_coeff.into(),
        ));
        params.push(ConfigParam::BlockContentMaxSize(
            self.block_content_max_size,
        ));
        for bft_leader in self.bft_leaders.iter() {
            params.push(ConfigParam::AddBftLeader(bft_leader.clone()));
        }
        params.push(ConfigParam::LinearFee(*self.linear_fees));
        params.push(ConfigParam::ProposalExpiration(self.proposal_expiration));

        match &self.reward_params {
            Some(p) => params.push(ConfigParam::RewardParams(p.clone())),
            None => (),
        };
        match &self.treasury_params {
            Some(p) => params.push(ConfigParam::TreasuryParams(*p)),
            None => (),
        };

        debug_assert_eq!(self, &Settings::new().apply(&params).unwrap());

        params
    }

    pub fn to_reward_params(&self) -> rewards::Parameters {
        let reward_drawing_limit_max = self.rewards_limit.clone();
        let pool_participation_capping = self.pool_participation_capping;

        match self.reward_params {
            None => rewards::Parameters::zero(),
            Some(RewardParams::Halving {
                constant,
                ratio,
                epoch_start,
                epoch_rate,
            }) => rewards::Parameters {
                initial_value: constant,
                compounding_ratio: ratio,
                compounding_type: rewards::CompoundingType::Halvening,
                epoch_start,
                epoch_rate,
                reward_drawing_limit_max,
                pool_participation_capping,
            },
            Some(RewardParams::Linear {
                constant,
                ratio,
                epoch_start,
                epoch_rate,
            }) => rewards::Parameters {
                initial_value: constant,
                compounding_ratio: ratio,
                compounding_type: rewards::CompoundingType::Linear,
                epoch_start,
                epoch_rate,
                reward_drawing_limit_max,
                pool_participation_capping,
            },
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{FeesGoesTo, Settings};
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for FeesGoesTo {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            if Arbitrary::arbitrary(g) {
                FeesGoesTo::Treasury
            } else {
                FeesGoesTo::Rewards
            }
        }
    }

    impl Arbitrary for Settings {
        fn arbitrary<G: Gen>(_: &mut G) -> Self {
            Settings::new()
        }
    }
}
