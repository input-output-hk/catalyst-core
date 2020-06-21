//! Mockchain ledger. Ledger exists in order to update the
//! current state and verify transactions.

use super::check::{self, TxVerifyError};
use super::governance::{Governance, ParametersGovernanceAction, TreasuryGovernanceAction};
use super::leaderlog::LeadersParticipationRecord;
use super::pots::Pots;
use super::reward_info::{EpochRewardsInfo, RewardsInfoParameters};
use crate::certificate::{PoolId, VoteAction, VotePlan};
use crate::chaineval::HeaderContentEvalContext;
use crate::chaintypes::{ChainLength, ConsensusType, HeaderId};
use crate::config::{self, ConfigParam};
use crate::date::{BlockDate, Epoch};
use crate::fee::{FeeAlgorithm, LinearFee};
use crate::fragment::{BlockContentHash, BlockContentSize, Contents, Fragment, FragmentId};
use crate::rewards;
use crate::setting::ActiveSlotsCoeffError;
use crate::stake::{
    PercentStake, PoolError, PoolStakeInformation, PoolsState, StakeControl, StakeDistribution,
};
use crate::transaction::*;
use crate::treasury::Treasury;
use crate::value::*;
use crate::vote::{CommitteeId, VotePlanLedger, VotePlanLedgerError, VotePlanStatus};
use crate::{account, certificate, legacy, multisig, setting, stake, update, utxo};
use chain_addr::{Address, Discrimination, Kind};
use chain_crypto::Verification;
use chain_time::{Epoch as TimeEpoch, SlotDuration, TimeEra, TimeFrame, Timeline};
use std::mem::swap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use thiserror::Error;

// static parameters, effectively this is constant in the parameter of the blockchain
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerStaticParameters {
    pub block0_initial_hash: HeaderId,
    pub block0_start_time: config::Block0Date,
    pub discrimination: Discrimination,
    pub kes_update_speed: u32,
}

// parameters to validate ledger
#[derive(Debug, Clone)]
pub struct LedgerParameters {
    /// Fees expected for transactions and certificates
    pub fees: LinearFee,
    /// Tax cut of the treasury which is applied straight after the reward pot
    /// is fully known
    pub treasury_tax: rewards::TaxType,
    /// Reward contribution parameters for this epoch
    pub reward_params: rewards::Parameters,
    /// the block content's max size in bytes
    pub block_content_max_size: BlockContentSize,
    /// the epoch stability parameter, the depth, number of blocks, to which
    /// we consider the blockchain to be stable and prevent rollback beyond
    /// that depth.
    pub epoch_stability_depth: u32,
    /// Where the fees get transfered to during the rewards
    pub fees_goes_to: setting::FeesGoesTo,
    /// List of committee members
    pub committees: Arc<Vec<CommitteeId>>,
}

/// Overall ledger structure.
///
/// This represent a given state related to utxo/old utxo/accounts/... at a given
/// point in time.
///
/// The ledger can be easily and cheaply cloned despite containing reference
/// to a lot of data (millions of utxos, thousands of accounts, ..)
#[derive(Clone, PartialEq, Eq)]
pub struct Ledger {
    pub(crate) utxos: utxo::Ledger<Address>,
    pub(crate) oldutxos: utxo::Ledger<legacy::OldAddress>,
    pub(crate) accounts: account::Ledger,
    pub(crate) settings: setting::Settings,
    pub(crate) updates: update::UpdateState,
    pub(crate) multisig: multisig::Ledger,
    pub(crate) delegation: PoolsState,
    pub(crate) static_params: Arc<LedgerStaticParameters>,
    pub(crate) date: BlockDate,
    pub(crate) chain_length: ChainLength,
    pub(crate) era: TimeEra,
    pub(crate) pots: Pots,
    pub(crate) leaders_log: LeadersParticipationRecord,
    pub(crate) votes: VotePlanLedger,
    pub(crate) governance: Governance,
}

// Dummy implementation of Debug for Ledger
impl std::fmt::Debug for Ledger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ledger")
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Block0Error {
    #[error("Transaction should not have inputs in a block0")]
    TransactionHasInput,
    #[error("Certificate should not have inputs in a block0")]
    CertTransactionHasInput,
    #[error("Certificate should not have outputs in a block0")]
    CertTransactionHasOutput,
    #[error("Transaction should not have witnesses in a block0")]
    TransactionHasWitnesses,
    #[error("The initial message is missing.")]
    InitialMessageMissing,
    #[error("Only one initial message is required")]
    InitialMessageMany,
    #[error("Block0 Date is duplicated in the initial message")]
    InitialMessageDuplicateBlock0Date,
    #[error("Address discrimination setting is duplicated in the initial fragment")]
    InitialMessageDuplicateDiscrimination,
    #[error("Consensus version is duplicated in the initial fragment")]
    InitialMessageDuplicateConsensusVersion,
    #[error("Slot Duration is duplicated in the initial fragment")]
    InitialMessageDuplicateSlotDuration,
    #[error("Epoch stability depth is duplicated in the initial fragment")]
    InitialMessageDuplicateEpochStabilityDepth,
    #[error("Praos active slot coefficient setting is duplicated in the initial fragment")]
    InitialMessageDuplicatePraosActiveSlotsCoeff,
    #[error("Missing block0 date in the initial fragment")]
    InitialMessageNoDate,
    #[error("Missing slot duration in the initial fragment")]
    InitialMessageNoSlotDuration,
    #[error("Missing slots per epoch in the initial fragment")]
    InitialMessageNoSlotsPerEpoch,
    #[error("Missing address discrimination in the initial fragment")]
    InitialMessageNoDiscrimination,
    #[error("Missing consensus version in the initial fragment")]
    InitialMessageNoConsensusVersion,
    #[error("Missing consensus leader id list in the initial fragment")]
    InitialMessageNoConsensusLeaderId,
    #[error("Missing praos active slot coefficient in the initial fragment")]
    InitialMessageNoPraosActiveSlotsCoeff,
    #[error("Missing KES Update speed in the initial fragment")]
    InitialMessageNoKesUpdateSpeed,
    #[error("Total initial value is too big")]
    UtxoTotalValueTooBig,
    #[error("Owner stake delegation are not valid in the block0")]
    HasOwnerStakeDelegation,
    #[error("Update proposal fragments are not valid in the block0")]
    HasUpdateProposal,
    #[error("Update vote fragments are not valid in the block0")]
    HasUpdateVote,
    #[error("Pool management are not valid in the block0")]
    HasPoolManagement,
    #[error("Vote casting are not valid in the block0")]
    HasVoteCast,
    #[error("Vote tallying are not valid in the block0")]
    HasVoteTally,
}

pub type OutputOldAddress = Output<legacy::OldAddress>;
pub type OutputAddress = Output<Address>;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Invalid settings")]
    Config(#[from] config::Error),
    #[error("The UTxO value ({expected}) in the transaction does not match the actually state value: {value}")]
    UtxoValueNotMatching { expected: Value, value: Value },
    #[error("Invalid UTxO")]
    UtxoError(#[from] utxo::Error),
    #[error("Transaction with invalid signature")]
    UtxoInvalidSignature {
        utxo: UtxoPointer,
        output: OutputAddress,
        witness: Witness,
    },
    #[error("Old Transaction with invalid signature")]
    OldUtxoInvalidSignature {
        utxo: UtxoPointer,
        output: OutputOldAddress,
        witness: Witness,
    },
    #[error("Old Transaction with invalid public key")]
    OldUtxoInvalidPublicKey {
        utxo: UtxoPointer,
        output: OutputOldAddress,
        witness: Witness,
    },
    #[error("Account with invalid signature")]
    AccountInvalidSignature {
        account: account::Identifier,
        witness: Witness,
    },
    #[error("Multisig with invalid signature")]
    MultisigInvalidSignature {
        multisig: multisig::Identifier,
        witness: Witness,
    },
    #[error("Transaction malformed")]
    TransactionMalformed(#[from] TxVerifyError),
    #[error("Error while computing the fees")]
    FeeCalculationError(#[from] ValueError),
    #[error("Praos active slot coefficient invalid: {error}")]
    PraosActiveSlotsCoeffInvalid { error: ActiveSlotsCoeffError },
    #[error("Failed to validate transaction balance")]
    TransactionBalanceInvalid(#[from] BalanceError),
    #[error("Invalid Block0")]
    Block0(#[from] Block0Error),
    #[error("Old UTxOs and Initial Message are not valid in a normal block")]
    Block0OnlyFragmentReceived,
    #[error("Error or Invalid account")]
    Account(#[from] account::LedgerError),
    #[error("Error or Invalid multisig")]
    Multisig(#[from] multisig::LedgerError),
    #[error("Inputs, outputs and fees are not balanced, transaction with {inputs} input and {outputs} output")]
    NotBalanced { inputs: Value, outputs: Value },
    #[error("Empty output")]
    ZeroOutput { output: Output<Address> },
    #[error("Output group invalid")]
    OutputGroupInvalid { output: Output<Address> },
    #[error("Error or Invalid delegation")]
    Delegation(#[from] PoolError),
    #[error("Invalid account identifier")]
    AccountIdentifierInvalid,
    #[error("Invalid discrimination")]
    InvalidDiscrimination,
    #[error("Expected an account witness")]
    ExpectingAccountWitness,
    #[error("Expected a UTxO witness")]
    ExpectingUtxoWitness,
    #[error("Expected an Initial Fragment")]
    ExpectingInitialMessage,
    #[error("Invalid certificate's signature")]
    CertificateInvalidSignature,
    #[error("Error or Invalid update")]
    Update(#[from] update::Error),
    #[error("Transaction for OwnerStakeDelegation is invalid. expecting 1 input, 1 witness and 0 output")]
    OwnerStakeDelegationInvalidTransaction,
    #[error("Transaction for VoteCast is invalid. expecting 1 input, 1 witness and 0 output")]
    VoteCastInvalidTransaction,
    #[error("Wrong chain length, expected {expected} but received {actual}")]
    WrongChainLength {
        actual: ChainLength,
        expected: ChainLength,
    },
    #[error("Non Monotonic date, chain date is at {chain_date} but the block is at {block_date}")]
    NonMonotonicDate {
        block_date: BlockDate,
        chain_date: BlockDate,
    },
    #[error("Wrong block content size, received {actual} bytes but max is {max} bytes")]
    InvalidContentSize { actual: u32, max: u32 },
    #[error("Wrong block content hash, received {actual} but expected {expected}")]
    InvalidContentHash {
        actual: BlockContentHash,
        expected: BlockContentHash,
    },
    #[error("Ledger cannot be reconstructed from serialized state because of missing entries")]
    IncompleteLedger,
    #[error("Ledger pot value invalid: {error}")]
    PotValueInvalid { error: ValueError },
    #[error("Pool registration with no owner")]
    PoolRegistrationHasNoOwner,
    #[error("Pool registration with too many owners")]
    PoolRegistrationHasTooManyOwners,
    #[error("Pool registration with too many operators")]
    PoolRegistrationHasTooManyOperators,
    #[error("Pool registration management threshold is zero")]
    PoolRegistrationManagementThresholdZero,
    #[error("Pool registration management threshold above owners")]
    PoolRegistrationManagementThresholdAbove,
    #[error("Pool Update not allowed yet")]
    PoolUpdateNotAllowedYet,
    #[error("Stake Delegation payload signature failed")]
    StakeDelegationSignatureFailed,
    #[error("Pool Retirement payload signature failed")]
    PoolRetirementSignatureFailed,
    #[error("Vote Plan Proof has an invalid signature")]
    VotePlanProofInvalidSignature,
    #[error("Vote Plan Proof ID is not present in the committee")]
    VotePlanProofInvalidCommittee,
    #[error("Vote plan contains proposal(s) that does not pass governance criteria")]
    VotePlanInvalidGovernanceParameters,
    #[error("Vote Tally Proof failed")]
    VoteTallyProofFailed,
    #[error("Pool update payload signature failed")]
    PoolUpdateSignatureFailed,
    #[error("Pool update last known registration hash doesn't match")]
    PoolUpdateLastHashDoesntMatch,
    #[error("Pool update doesnt currently allow fees update")]
    PoolUpdateFeesNotAllowedYet,
    #[error("Update not yet allowed")]
    UpdateNotAllowedYet,
    #[error("Cannot add the vote plan")]
    CannotAddVotePlan(
        #[from]
        #[source]
        VotePlanLedgerError,
    ),
    #[error("Scripts addresses are not yet supported by the system")]
    ScriptsAddressNotAllowedYet,
}

impl LedgerParameters {
    pub fn treasury_tax(&self) -> rewards::TaxType {
        self.treasury_tax
    }
}

impl Ledger {
    fn empty(
        settings: setting::Settings,
        static_params: LedgerStaticParameters,
        era: TimeEra,
        pots: Pots,
    ) -> Self {
        Ledger {
            utxos: utxo::Ledger::new(),
            oldutxos: utxo::Ledger::new(),
            accounts: account::Ledger::new(),
            settings,
            updates: update::UpdateState::new(),
            multisig: multisig::Ledger::new(),
            delegation: PoolsState::new(),
            static_params: Arc::new(static_params),
            date: BlockDate::first(),
            chain_length: ChainLength(0),
            era,
            pots,
            leaders_log: LeadersParticipationRecord::new(),
            votes: VotePlanLedger::new(),
            governance: Governance::default(),
        }
    }

    pub fn new<'a, I>(block0_initial_hash: HeaderId, contents: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = &'a Fragment>,
    {
        let mut content_iter = contents.into_iter();

        let init_ents = match content_iter.next() {
            Some(Fragment::Initial(ref init_ents)) => Ok(init_ents),
            Some(_) => Err(Error::ExpectingInitialMessage),
            None => Err(Error::Block0(Block0Error::InitialMessageMissing)),
        }?;

        let mut ledger = {
            let mut regular_ents = crate::fragment::ConfigParams::new();
            let mut block0_start_time = None;
            let mut slot_duration = None;
            let mut discrimination = None;
            let mut slots_per_epoch = None;
            let mut kes_update_speed = None;
            let mut pots = Pots::zero();

            for param in init_ents.iter() {
                match param {
                    ConfigParam::Block0Date(d) => {
                        block0_start_time = Some(*d);
                    }
                    ConfigParam::Discrimination(d) => {
                        discrimination = Some(*d);
                    }
                    ConfigParam::SlotDuration(d) => {
                        slot_duration = Some(*d);
                    }
                    ConfigParam::SlotsPerEpoch(n) => {
                        slots_per_epoch = Some(*n);
                    }
                    ConfigParam::KESUpdateSpeed(n) => {
                        kes_update_speed = Some(*n);
                    }
                    ConfigParam::TreasuryAdd(v) => {
                        pots.treasury = Treasury::initial(*v);
                    }
                    ConfigParam::RewardPot(v) => {
                        pots.rewards = *v;
                    }
                    _ => regular_ents.push(param.clone()),
                }
            }

            // here we make sure those specific parameters are present, otherwise we returns a given error
            let block0_start_time =
                block0_start_time.ok_or(Error::Block0(Block0Error::InitialMessageNoDate))?;
            let discrimination =
                discrimination.ok_or(Error::Block0(Block0Error::InitialMessageNoDiscrimination))?;
            let slot_duration =
                slot_duration.ok_or(Error::Block0(Block0Error::InitialMessageNoSlotDuration))?;
            let slots_per_epoch =
                slots_per_epoch.ok_or(Error::Block0(Block0Error::InitialMessageNoSlotsPerEpoch))?;
            let kes_update_speed = kes_update_speed
                .ok_or(Error::Block0(Block0Error::InitialMessageNoKesUpdateSpeed))?;

            let static_params = LedgerStaticParameters {
                block0_initial_hash,
                block0_start_time,
                discrimination,
                kes_update_speed,
            };

            let system_time = SystemTime::UNIX_EPOCH + Duration::from_secs(block0_start_time.0);
            let timeline = Timeline::new(system_time);
            let tf = TimeFrame::new(timeline, SlotDuration::from_secs(slot_duration as u32));
            let slot0 = tf.slot0();

            let era = TimeEra::new(slot0, TimeEpoch(0), slots_per_epoch);

            let settings = setting::Settings::new().apply(&regular_ents)?;

            if settings.bft_leaders.is_empty() {
                return Err(Error::Block0(
                    Block0Error::InitialMessageNoConsensusLeaderId,
                ));
            }
            Ledger::empty(settings, static_params, era, pots)
        };

        let params = ledger.get_ledger_parameters();

        for content in content_iter {
            let fragment_id = content.hash();
            match content {
                Fragment::Initial(_) => {
                    return Err(Error::Block0(Block0Error::InitialMessageMany));
                }
                Fragment::OldUtxoDeclaration(old) => {
                    ledger.oldutxos = apply_old_declaration(&fragment_id, ledger.oldutxos, old)?;
                }
                Fragment::Transaction(tx) => {
                    let tx = tx.as_slice();
                    check::valid_block0_transaction_no_inputs(&tx)?;

                    ledger = ledger.apply_tx_outputs(fragment_id, tx.outputs())?;
                }
                Fragment::UpdateProposal(_) => {
                    return Err(Error::Block0(Block0Error::HasUpdateProposal));
                }
                Fragment::UpdateVote(_) => {
                    return Err(Error::Block0(Block0Error::HasUpdateVote));
                }
                Fragment::OwnerStakeDelegation(_) => {
                    return Err(Error::Block0(Block0Error::HasOwnerStakeDelegation));
                }
                Fragment::StakeDelegation(tx) => {
                    let tx = tx.as_slice();
                    check::valid_block0_cert_transaction(&tx)?;
                    ledger = ledger.apply_stake_delegation(&tx.payload().into_payload())?;
                }
                Fragment::PoolRegistration(tx) => {
                    let tx = tx.as_slice();
                    check::valid_block0_cert_transaction(&tx)?;
                    ledger = ledger.apply_pool_registration(&tx.payload().into_payload())?;
                }
                Fragment::PoolRetirement(_) => {
                    return Err(Error::Block0(Block0Error::HasPoolManagement));
                }
                Fragment::PoolUpdate(_) => {
                    return Err(Error::Block0(Block0Error::HasPoolManagement));
                }
                Fragment::VotePlan(tx) => {
                    let tx = tx.as_slice();
                    check::valid_block0_cert_transaction(&tx)?;
                    // here current date is the date of the previous state of the
                    // ledger. It makes sense only because we are creating the block0
                    let cur_date = ledger.date();
                    ledger = ledger.apply_vote_plan(
                        &tx,
                        cur_date,
                        tx.payload().into_payload(),
                        &params,
                        tx.payload_auth().into_payload_auth(),
                    )?;
                }
                Fragment::VoteCast(_) => {
                    return Err(Error::Block0(Block0Error::HasVoteCast));
                }
                Fragment::VoteTally(_) => {
                    return Err(Error::Block0(Block0Error::HasVoteTally));
                }
            }
        }

        ledger.validate_utxo_total_value()?;
        Ok(ledger)
    }

    pub fn can_distribute_reward(&self) -> bool {
        self.leaders_log.total() != 0
    }

    pub fn apply_protocol_changes(&self) -> Result<Self, Error> {
        let mut new = self.clone();

        for action in new.governance.parameters.logs() {
            match action {
                ParametersGovernanceAction::NoOp => {}
                ParametersGovernanceAction::RewardAdd { value } => {
                    new.pots.rewards_add(*value)?;
                }
            }
        }

        new.governance.parameters.logs_clear();
        Ok(new)
    }

    /// This need to be called before the *first* block of a new epoch
    ///
    /// * Reset the leaders log
    /// * Distribute the contribution (rewards + fees) to pools and their delegatees
    pub fn distribute_rewards<'a>(
        &'a self,
        distribution: &StakeDistribution,
        ledger_params: &LedgerParameters,
        rewards_info_params: RewardsInfoParameters,
    ) -> Result<(Self, EpochRewardsInfo), Error> {
        let mut new_ledger = self.clone();
        let mut rewards_info = EpochRewardsInfo::new(rewards_info_params);

        if self.leaders_log.total() == 0 {
            return Ok((new_ledger, rewards_info));
        }

        let treasury_initial_value = new_ledger.pots.treasury_value();

        // grab the total contribution in the system
        // with all the stake pools and start rewarding them

        let epoch = new_ledger.date.epoch + 1;

        let system_info = rewards::SystemInformation {
            declared_stake: distribution.get_total_stake(),
        };

        let expected_epoch_reward = rewards::rewards_contribution_calculation(
            epoch,
            &ledger_params.reward_params,
            &system_info,
        );

        let drawn = new_ledger.pots.draw_reward(expected_epoch_reward);

        // set basic reward info
        {
            let fees_in_pot = new_ledger.pots.fees_value();
            rewards_info.drawn = drawn;
            rewards_info.fees = fees_in_pot;
        }

        let mut total_reward = drawn;

        // Move fees in the rewarding pots for distribution or depending on settings
        // to the treasury directly
        match ledger_params.fees_goes_to {
            setting::FeesGoesTo::Rewards => {
                total_reward = (total_reward + new_ledger.pots.siphon_fees()).unwrap();
            }
            setting::FeesGoesTo::Treasury => {
                let fees = new_ledger.pots.siphon_fees();
                new_ledger.pots.treasury_add(fees)?
            }
        }

        // Take treasury cut
        total_reward = {
            let treasury_distr = rewards::tax_cut(total_reward, &ledger_params.treasury_tax)?;
            new_ledger.pots.treasury_add(treasury_distr.taxed)?;
            treasury_distr.after_tax
        };

        // distribute the rest to all leaders now
        let mut leaders_log = LeadersParticipationRecord::new();
        swap(&mut new_ledger.leaders_log, &mut leaders_log);

        if total_reward > Value::zero() {
            // pool capping only exists if there's enough participants
            let pool_capper = match ledger_params.reward_params.pool_participation_capping {
                None => None,
                Some((threshold, expected_nb_pools)) => {
                    let nb_participants = leaders_log.nb_participants();
                    if nb_participants >= threshold.get() as usize {
                        Some(Value(total_reward.0 / expected_nb_pools.get() as u64))
                    } else {
                        None
                    }
                }
            };

            let total_blocks = leaders_log.total();
            let reward_unit = total_reward.split_in(total_blocks);

            for (pool_id, pool_blocks) in leaders_log.iter() {
                // possibly cap the reward for a given pool.
                // if this is capped, then the overflow amount is send to treasury
                let pool_total_reward_uncapped = reward_unit.parts.scale(*pool_blocks).unwrap();
                let pool_total_reward = match pool_capper {
                    None => pool_total_reward_uncapped,
                    Some(pool_cap) => {
                        let actual_pool_total = std::cmp::min(pool_cap, pool_total_reward_uncapped);
                        let forfeited = (pool_total_reward_uncapped - actual_pool_total).unwrap();
                        new_ledger.pots.treasury_add(forfeited)?;
                        actual_pool_total
                    }
                };

                match distribution.to_pools.get(pool_id) {
                    Some(pool_distribution) => {
                        new_ledger.distribute_poolid_rewards(
                            &mut rewards_info,
                            epoch,
                            &pool_id,
                            pool_total_reward,
                            pool_distribution,
                        )?;
                    }
                    _ => {
                        // dump reward to treasury
                        new_ledger.pots.treasury_add(pool_total_reward)?;
                    }
                }
            }

            if reward_unit.remaining > Value::zero() {
                // if anything remaining, put it in treasury
                new_ledger.pots.treasury_add(reward_unit.remaining)?;
            }
        }

        let treasury_added_value =
            (new_ledger.pots.treasury_value() - treasury_initial_value).unwrap();
        rewards_info.set_treasury(treasury_added_value);

        Ok((new_ledger, rewards_info))
    }

    fn distribute_poolid_rewards(
        &mut self,
        reward_info: &mut EpochRewardsInfo,
        epoch: Epoch,
        pool_id: &PoolId,
        total_reward: Value,
        distribution: &PoolStakeInformation,
    ) -> Result<(), Error> {
        let reg = match distribution.registration {
            None => {
                self.pots.treasury_add(total_reward)?;
                return Ok(());
            }
            Some(ref reg) => reg,
        };

        let distr = rewards::tax_cut(total_reward, &reg.rewards).unwrap();

        reward_info.set_stake_pool(pool_id, distr.taxed, distr.after_tax);
        self.delegation
            .stake_pool_set_rewards(pool_id, epoch, distr.taxed, distr.after_tax)?;

        // distribute to pool owners (or the reward account)
        match &reg.reward_account {
            Some(reward_account) => match reward_account {
                AccountIdentifier::Single(single_account) => {
                    self.accounts = self.accounts.add_rewards_to_account(
                        &single_account,
                        epoch,
                        distr.taxed,
                        (),
                    )?;
                    reward_info.add_to_account(&single_account, distr.taxed);
                }
                AccountIdentifier::Multi(_multi_account) => unimplemented!(),
            },
            None => {
                if reg.owners.len() > 1 {
                    let splitted = distr.taxed.split_in(reg.owners.len() as u32);
                    for owner in &reg.owners {
                        let id = owner.clone().into();
                        self.accounts =
                            self.accounts
                                .add_rewards_to_account(&id, epoch, splitted.parts, ())?;
                        reward_info.add_to_account(&id, splitted.parts);
                    }
                    // pool owners 0 get potentially an extra sweetener of value 1 to #owners - 1
                    if splitted.remaining > Value::zero() {
                        let id = reg.owners[0].clone().into();
                        self.accounts = self.accounts.add_rewards_to_account(
                            &id,
                            epoch,
                            splitted.remaining,
                            (),
                        )?;
                        reward_info.add_to_account(&id, splitted.remaining);
                    }
                } else {
                    let id = reg.owners[0].clone().into();
                    self.accounts =
                        self.accounts
                            .add_rewards_to_account(&id, epoch, distr.taxed, ())?;
                    reward_info.add_to_account(&id, distr.taxed);
                }
            }
        }

        // distribute the rest to delegators
        let mut leftover_reward = distr.after_tax;
        if leftover_reward > Value::zero() {
            for (account, stake) in distribution.stake.accounts.iter() {
                let ps = PercentStake::new(*stake, distribution.stake.total);
                let r = ps.scale_value(distr.after_tax);
                leftover_reward = (leftover_reward - r).unwrap();
                self.accounts = self
                    .accounts
                    .add_rewards_to_account(account, epoch, r, ())?;
                reward_info.add_to_account(account, r);
            }
        }

        if leftover_reward > Value::zero() {
            self.pots.treasury_add(leftover_reward)?;
        }

        Ok(())
    }

    /// Try to apply messages to a State, and return the new State if successful
    pub fn apply_block(
        &self,
        ledger_params: &LedgerParameters,
        contents: &Contents,
        metadata: &HeaderContentEvalContext,
    ) -> Result<Self, Error> {
        let mut new_ledger = self.clone();

        new_ledger.chain_length = self.chain_length.increase();

        let (content_hash, content_size) = contents.compute_hash_size();

        if content_size > ledger_params.block_content_max_size {
            return Err(Error::InvalidContentSize {
                actual: content_size,
                max: ledger_params.block_content_max_size,
            });
        }

        if content_hash != metadata.content_hash {
            return Err(Error::InvalidContentHash {
                actual: content_hash,
                expected: metadata.content_hash,
            });
        }

        // Check if the metadata (date/heigth) check out compared to the current state
        if metadata.chain_length != new_ledger.chain_length {
            return Err(Error::WrongChainLength {
                actual: metadata.chain_length,
                expected: new_ledger.chain_length,
            });
        }

        if metadata.block_date <= new_ledger.date {
            return Err(Error::NonMonotonicDate {
                block_date: metadata.block_date,
                chain_date: new_ledger.date,
            });
        }

        // double check that if we had an epoch transition, distribute_rewards has been called
        if metadata.block_date.epoch > new_ledger.date.epoch && self.leaders_log.total() > 0 {
            panic!("internal error: apply_block called after epoch transition, but distribute_rewards has not been called")
        }

        // Process Update proposals if needed
        let (updates, settings) = new_ledger.updates.process_proposals(
            new_ledger.settings,
            new_ledger.date,
            metadata.block_date,
        )?;
        new_ledger.updates = updates;
        new_ledger.settings = settings;

        // Apply all the fragments
        for content in contents.iter() {
            new_ledger = new_ledger.apply_fragment(ledger_params, content, metadata.block_date)?;
        }

        // Update the ledger metadata related to eval context
        new_ledger.date = metadata.block_date;
        match metadata.gp_content {
            None => {}
            Some(ref gp_content) => {
                new_ledger
                    .settings
                    .consensus_nonce
                    .hash_with(&gp_content.nonce);
                new_ledger
                    .leaders_log
                    .increase_for(&gp_content.pool_creator);
            }
        };

        Ok(new_ledger)
    }

    /// Try to apply a message to the State, and return the new State if successful
    ///
    /// this does not _advance_ the state to the new _state_ but apply a simple fragment
    /// of block to the current context.
    ///
    pub fn apply_fragment(
        &self,
        ledger_params: &LedgerParameters,
        content: &Fragment,
        block_date: BlockDate,
    ) -> Result<Self, Error> {
        let mut new_ledger = self.clone();

        let fragment_id = content.hash();
        match content {
            Fragment::Initial(_) => return Err(Error::Block0OnlyFragmentReceived),
            Fragment::OldUtxoDeclaration(_) => return Err(Error::Block0OnlyFragmentReceived),
            Fragment::Transaction(tx) => {
                let tx = tx.as_slice();
                let (new_ledger_, _fee) =
                    new_ledger.apply_transaction(&fragment_id, &tx, &ledger_params)?;
                new_ledger = new_ledger_;
            }
            Fragment::OwnerStakeDelegation(tx) => {
                let tx = tx.as_slice();
                let (new_ledger_, _fee) =
                    new_ledger.apply_owner_stake_delegation(&tx, &ledger_params)?;
                new_ledger = new_ledger_;
            }
            Fragment::StakeDelegation(tx) => {
                let tx = tx.as_slice();
                let payload = tx.payload().into_payload();
                let payload_auth = tx.payload_auth().into_payload_auth();
                let verified = match payload_auth {
                    AccountBindingSignature::Single(signature) => {
                        let account_pk = payload
                            .account_id
                            .to_single_account()
                            .ok_or(Error::AccountIdentifierInvalid)?;
                        signature
                            .verify_slice(&account_pk.into(), &tx.transaction_binding_auth_data())
                    }
                    AccountBindingSignature::Multi(_) => {
                        // TODO
                        Verification::Failed
                    }
                };

                if verified == Verification::Failed {
                    return Err(Error::StakeDelegationSignatureFailed);
                }

                let (new_ledger_, _fee) =
                    new_ledger.apply_transaction(&fragment_id, &tx, &ledger_params)?;
                new_ledger = new_ledger_.apply_stake_delegation(&payload)?;
            }
            Fragment::PoolRegistration(tx) => {
                let tx = tx.as_slice();
                let (new_ledger_, _fee) =
                    new_ledger.apply_transaction(&fragment_id, &tx, &ledger_params)?;
                new_ledger = new_ledger_.apply_pool_registration_signcheck(
                    &tx.payload().into_payload(),
                    &tx.transaction_binding_auth_data(),
                    tx.payload_auth().into_payload_auth(),
                )?;
            }
            Fragment::PoolRetirement(tx) => {
                let tx = tx.as_slice();

                let (new_ledger_, _fee) =
                    new_ledger.apply_transaction(&fragment_id, &tx, &ledger_params)?;
                new_ledger = new_ledger_.apply_pool_retirement(
                    &tx.payload().into_payload(),
                    &tx.transaction_binding_auth_data(),
                    tx.payload_auth().into_payload_auth(),
                )?;
            }
            Fragment::PoolUpdate(tx) => {
                let tx = tx.as_slice();

                let (new_ledger_, _fee) =
                    new_ledger.apply_transaction(&fragment_id, &tx, &ledger_params)?;
                new_ledger = new_ledger_.apply_pool_update(
                    &tx.payload().into_payload(),
                    &tx.transaction_binding_auth_data(),
                    tx.payload_auth().into_payload_auth(),
                )?;
            }
            Fragment::UpdateProposal(update_proposal) => {
                if true {
                    return Err(Error::UpdateNotAllowedYet);
                }
                new_ledger =
                    new_ledger.apply_update_proposal(fragment_id, &update_proposal, block_date)?;
            }
            Fragment::UpdateVote(vote) => {
                if true {
                    return Err(Error::UpdateNotAllowedYet);
                }
                new_ledger = new_ledger.apply_update_vote(&vote)?;
            }
            Fragment::VotePlan(tx) => {
                let tx = tx.as_slice();
                let (new_ledger_, _fee) =
                    new_ledger.apply_transaction(&fragment_id, &tx, &ledger_params)?;
                new_ledger = new_ledger_.apply_vote_plan(
                    &tx,
                    block_date,
                    tx.payload().into_payload(),
                    &ledger_params,
                    tx.payload_auth().into_payload_auth(),
                )?;
            }
            Fragment::VoteCast(tx) => {
                let tx = tx.as_slice();
                let (new_ledger_, _fee) = new_ledger.apply_vote_cast(&tx, &ledger_params)?;
                new_ledger = new_ledger_;
            }
            Fragment::VoteTally(tx) => {
                let tx = tx.as_slice();

                let (new_ledger_, _fee) =
                    new_ledger.apply_transaction(&fragment_id, &tx, &ledger_params)?;

                new_ledger = new_ledger_.apply_vote_tally(
                    &tx.payload().into_payload(),
                    &tx.transaction_binding_auth_data(),
                    tx.payload_auth().into_payload_auth(),
                )?;
            }
        }

        Ok(new_ledger)
    }

    pub fn apply_transaction<'a, Extra>(
        mut self,
        fragment_id: &FragmentId,
        tx: &TransactionSlice<'a, Extra>,
        dyn_params: &LedgerParameters,
    ) -> Result<(Self, Value), Error>
    where
        Extra: Payload,
        LinearFee: FeeAlgorithm,
    {
        check::valid_transaction_ios_number(tx)?;
        let fee = calculate_fee(tx, dyn_params);
        tx.verify_strictly_balanced(fee)?;
        self = self.apply_tx_inputs(tx)?;
        self = self.apply_tx_outputs(*fragment_id, tx.outputs())?;
        self = self.apply_tx_fee(fee)?;
        Ok((self, fee))
    }

    pub fn apply_update(mut self, update: &update::UpdateProposal) -> Result<Self, Error> {
        self.settings = self.settings.apply(&update.changes)?;
        Ok(self)
    }

    pub fn apply_update_proposal(
        mut self,
        proposal_id: update::UpdateProposalId,
        proposal: &update::SignedUpdateProposal,
        cur_date: BlockDate,
    ) -> Result<Self, Error> {
        self.updates =
            self.updates
                .apply_proposal(proposal_id, proposal, &self.settings, cur_date)?;
        Ok(self)
    }

    pub fn apply_update_vote(mut self, vote: &update::SignedUpdateVote) -> Result<Self, Error> {
        self.updates = self.updates.apply_vote(vote, &self.settings)?;
        Ok(self)
    }

    pub fn apply_vote_plan<'a>(
        mut self,
        tx: &TransactionSlice<'a, VotePlan>,
        cur_date: BlockDate,
        vote_plan: VotePlan,
        dyn_params: &LedgerParameters,
        sig: certificate::VotePlanProof,
    ) -> Result<Self, Error> {
        if sig.verify(&tx.transaction_binding_auth_data()) == Verification::Failed {
            return Err(Error::VotePlanProofInvalidSignature);
        }

        if !vote_plan.check_governance(&self.governance) {
            return Err(Error::VotePlanInvalidGovernanceParameters);
        }

        let committee: std::collections::HashSet<CommitteeId> = {
            let mut vec = Vec::with_capacity(tx.nb_inputs() as usize);

            if !vote_plan.is_governance() {
                for input in tx.inputs().iter() {
                    match input.to_enum() {
                        InputEnum::UtxoInput(_) => {
                            return Err(Error::VoteCastInvalidTransaction);
                        }
                        InputEnum::AccountInput(account_id, _value) => {
                            use std::convert::TryInto as _;

                            vec.push(account_id.as_ref().try_into().unwrap());
                        }
                    }
                }
            }

            vec.into_iter()
                .chain(dyn_params.committees.iter().cloned())
                .collect()
        };

        if !committee.contains(&sig.id) {
            return Err(Error::VotePlanProofInvalidCommittee);
        }

        self.votes = self.votes.add_vote_plan(cur_date, vote_plan, committee)?;
        Ok(self)
    }

    pub fn apply_vote_cast<'a>(
        mut self,
        tx: &TransactionSlice<'a, certificate::VoteCast>,
        dyn_params: &LedgerParameters,
    ) -> Result<(Self, Value), Error> {
        let sign_data_hash = tx.transaction_sign_data_hash();

        let (account_id, value, witness) = {
            check::valid_vote_cast(tx)?;

            let input = tx.inputs().iter().next().unwrap();
            match input.to_enum() {
                InputEnum::UtxoInput(_) => {
                    return Err(Error::VoteCastInvalidTransaction);
                }
                InputEnum::AccountInput(account_id, value) => {
                    let witness = tx.witnesses().iter().next().unwrap();
                    (account_id, value, witness)
                }
            }
        };

        let fee = dyn_params.fees.calculate_tx(tx);
        if fee != value {
            return Err(Error::NotBalanced {
                inputs: value,
                outputs: fee,
            });
        }

        match match_identifier_witness(&account_id, &witness)? {
            MatchingIdentifierWitness::Single(account_id, witness) => {
                self.accounts = input_single_account_verify(
                    self.accounts,
                    &self.static_params.block0_initial_hash,
                    &sign_data_hash,
                    &account_id,
                    witness,
                    value,
                )?;
            }
            MatchingIdentifierWitness::Multi(account_id, witness) => {
                self.multisig = input_multi_account_verify(
                    self.multisig,
                    &self.static_params.block0_initial_hash,
                    &sign_data_hash,
                    &account_id,
                    witness,
                    value,
                )?;
            }
        };
        self = self.apply_tx_fee(fee)?;

        let vote = tx.payload().into_payload();
        self.votes = self.votes.apply_vote(self.date(), account_id, vote)?;

        Ok((self, fee))
    }

    pub fn active_vote_plans(&self) -> Vec<VotePlanStatus> {
        self.votes
            .plans
            .iter()
            .map(|(_, plan)| plan.statuses())
            .collect()
    }

    pub fn apply_vote_tally<'a>(
        mut self,
        tally: &certificate::VoteTally,
        bad: &TransactionBindingAuthData<'a>,
        sig: certificate::TallyProof,
    ) -> Result<Self, Error> {
        if sig.verify(tally, bad) == Verification::Failed {
            return Err(Error::VoteTallyProofFailed);
        }

        let stake = StakeControl::new_with(&self.accounts, &self.utxos);

        let mut actions = Vec::new();

        let mut f = |action: &VoteAction| actions.push(action.clone());

        self.votes = self.votes.apply_committee_result(
            self.date(),
            &stake,
            &self.governance,
            tally,
            sig,
            &mut f,
        )?;

        for action in actions {
            match action {
                VoteAction::OffChain => {}
                VoteAction::Treasury {
                    action: TreasuryGovernanceAction::NoOp,
                } => {}
                VoteAction::Treasury {
                    action: TreasuryGovernanceAction::TransferToRewards { value },
                } => {
                    let value = self.pots.draw_treasury(value);
                    self.pots.rewards_add(value)?;
                }
                VoteAction::Parameters { action } => {
                    if self.governance.parameters.logs_register(action).is_err() {
                        unimplemented!("the action was already recorded for this epoch")
                    } else {
                        // nothing
                    }
                }
            }
        }

        Ok(self)
    }

    pub fn apply_pool_registration_signcheck<'a>(
        self,
        cert: &certificate::PoolRegistration,
        bad: &TransactionBindingAuthData<'a>,
        sig: certificate::PoolSignature,
    ) -> Result<Self, Error> {
        check::valid_pool_registration_certificate(cert)?;
        check::valid_pool_signature(&sig)?;

        if sig.verify(cert, bad) == Verification::Failed {
            return Err(Error::PoolRetirementSignatureFailed);
        }

        self.apply_pool_registration(cert)
    }

    pub fn apply_pool_registration(
        mut self,
        cert: &certificate::PoolRegistration,
    ) -> Result<Self, Error> {
        check::valid_pool_registration_certificate(cert)?;

        self.delegation = self.delegation.register_stake_pool(cert.clone())?;
        Ok(self)
    }

    pub fn apply_pool_retirement<'a>(
        mut self,
        auth_cert: &certificate::PoolRetirement,
        bad: &TransactionBindingAuthData<'a>,
        sig: certificate::PoolSignature,
    ) -> Result<Self, Error> {
        check::valid_pool_retirement_certificate(auth_cert)?;
        check::valid_pool_signature(&sig)?;

        let reg = self.delegation.stake_pool_get(&auth_cert.pool_id)?;
        if sig.verify(reg, bad) == Verification::Failed {
            return Err(Error::PoolRetirementSignatureFailed);
        }

        self.delegation = self.delegation.deregister_stake_pool(&auth_cert.pool_id)?;
        Ok(self)
    }

    pub fn apply_pool_update<'a>(
        mut self,
        auth_cert: &certificate::PoolUpdate,
        bad: &TransactionBindingAuthData<'a>,
        sig: certificate::PoolSignature,
    ) -> Result<Self, Error> {
        check::valid_pool_update_certificate(auth_cert)?;
        check::valid_pool_signature(&sig)?;

        let state = self.delegation.stake_pool_get_state(&auth_cert.pool_id)?;

        if auth_cert.last_pool_reg_hash != state.current_pool_registration_hash() {
            return Err(Error::PoolUpdateLastHashDoesntMatch);
        }

        let new = &auth_cert.new_pool_reg;

        // don't allow any fees update for now
        if new.rewards != state.registration.rewards {
            return Err(Error::PoolUpdateFeesNotAllowedYet);
        }

        if sig.verify(&state.registration, bad) == Verification::Failed {
            return Err(Error::PoolUpdateSignatureFailed);
        }

        let new = new.clone();

        let mut updated_state = state.clone();
        updated_state.registration = Arc::new(new);

        self.delegation
            .stake_pool_set_state(&auth_cert.pool_id, updated_state)?;

        Ok(self)
    }

    pub fn apply_stake_delegation(
        mut self,
        auth_cert: &certificate::StakeDelegation,
    ) -> Result<Self, Error> {
        let delegation = &auth_cert.delegation;

        let account_key = auth_cert
            .account_id
            .to_single_account()
            .ok_or(Error::AccountIdentifierInvalid)?;
        self.accounts = self.accounts.set_delegation(&account_key, delegation)?;
        Ok(self)
    }

    pub fn apply_owner_stake_delegation<'a>(
        mut self,
        tx: &TransactionSlice<'a, certificate::OwnerStakeDelegation>,
        dyn_params: &LedgerParameters,
    ) -> Result<(Self, Value), Error> {
        let sign_data_hash = tx.transaction_sign_data_hash();

        let (account_id, value, witness) = {
            check::valid_stake_owner_delegation_transaction(tx)?;

            let input = tx.inputs().iter().next().unwrap();
            match input.to_enum() {
                InputEnum::UtxoInput(_) => {
                    return Err(Error::OwnerStakeDelegationInvalidTransaction);
                }
                InputEnum::AccountInput(account_id, value) => {
                    let witness = tx.witnesses().iter().next().unwrap();
                    (account_id, value, witness)
                }
            }
        };

        let fee = dyn_params.fees.calculate_tx(tx);
        if fee != value {
            return Err(Error::NotBalanced {
                inputs: value,
                outputs: fee,
            });
        }

        match match_identifier_witness(&account_id, &witness)? {
            MatchingIdentifierWitness::Single(account_id, witness) => {
                let single = input_single_account_verify(
                    self.accounts,
                    &self.static_params.block0_initial_hash,
                    &sign_data_hash,
                    &account_id,
                    witness,
                    value,
                )?;
                self.accounts = single.set_delegation(
                    &account_id,
                    tx.payload().into_payload().get_delegation_type(),
                )?;
            }
            MatchingIdentifierWitness::Multi(account_id, witness) => {
                let multi = input_multi_account_verify(
                    self.multisig,
                    &self.static_params.block0_initial_hash,
                    &sign_data_hash,
                    &account_id,
                    witness,
                    value,
                )?;
                self.multisig = multi.set_delegation(
                    &account_id,
                    tx.payload().into_payload().get_delegation_type(),
                )?;
            }
        };
        self = self.apply_tx_fee(fee)?;
        Ok((self, fee))
    }

    pub fn get_stake_distribution(&self) -> StakeDistribution {
        stake::get_distribution(&self.accounts, &self.delegation, &self.utxos)
    }

    /// access the ledger static parameters
    pub fn get_static_parameters(&self) -> &LedgerStaticParameters {
        self.static_params.as_ref()
    }

    pub fn accounts(&self) -> &account::Ledger {
        &self.accounts
    }

    pub fn get_ledger_parameters(&self) -> LedgerParameters {
        LedgerParameters {
            fees: *self.settings.linear_fees,
            treasury_tax: self
                .settings
                .treasury_params
                .unwrap_or_else(rewards::TaxType::zero),
            reward_params: self.settings.to_reward_params(),
            block_content_max_size: self.settings.block_content_max_size,
            epoch_stability_depth: self.settings.epoch_stability_depth,
            fees_goes_to: self.settings.fees_goes_to,
            committees: Arc::clone(&self.settings.committees),
        }
    }

    pub fn consensus_version(&self) -> ConsensusType {
        self.settings.consensus_version
    }

    pub fn utxo_out(
        &self,
        fragment_id: FragmentId,
        index: TransactionIndex,
    ) -> Option<&Output<Address>> {
        self.utxos
            .get(&fragment_id, index)
            .map(|entry| entry.output)
    }

    pub fn utxos(&self) -> utxo::Iter<'_, Address> {
        self.utxos.iter()
    }

    pub fn chain_length(&self) -> ChainLength {
        self.chain_length
    }

    pub fn settings(&mut self) -> &mut setting::Settings {
        &mut self.settings
    }

    pub fn delegation(&self) -> &PoolsState {
        &self.delegation
    }

    pub fn delegation_mut(&mut self) -> &mut PoolsState {
        &mut self.delegation
    }

    pub fn date(&self) -> BlockDate {
        self.date
    }

    pub fn era(&self) -> &TimeEra {
        &self.era
    }

    fn validate_utxo_total_value(&self) -> Result<(), Error> {
        self.get_total_value()?;
        Ok(())
    }

    pub fn get_total_value(&self) -> Result<Value, Error> {
        let old_utxo_values = self.oldutxos.iter().map(|entry| entry.output.value);
        let new_utxo_values = self.utxos.iter().map(|entry| entry.output.value);
        let account_value = self
            .accounts
            .get_total_value()
            .map_err(|_| Error::Block0(Block0Error::UtxoTotalValueTooBig))?;
        let multisig_value = self
            .multisig
            .get_total_value()
            .map_err(|_| Error::Block0(Block0Error::UtxoTotalValueTooBig))?;
        let all_utxo_values = old_utxo_values
            .chain(new_utxo_values)
            .chain(Some(account_value))
            .chain(Some(multisig_value))
            .chain(self.pots.values());
        Value::sum(all_utxo_values).map_err(|_| Error::Block0(Block0Error::UtxoTotalValueTooBig))
    }

    fn apply_tx_inputs<'a, Extra: Payload>(
        mut self,
        tx: &TransactionSlice<'a, Extra>,
    ) -> Result<Self, Error> {
        let sign_data_hash = tx.transaction_sign_data_hash();
        for (input, witness) in tx.inputs_and_witnesses().iter() {
            match input.to_enum() {
                InputEnum::UtxoInput(utxo) => {
                    self = self.apply_input_to_utxo(&sign_data_hash, &utxo, &witness)?
                }
                InputEnum::AccountInput(account_id, value) => {
                    match match_identifier_witness(&account_id, &witness)? {
                        MatchingIdentifierWitness::Single(account_id, witness) => {
                            self.accounts = input_single_account_verify(
                                self.accounts,
                                &self.static_params.block0_initial_hash,
                                &sign_data_hash,
                                &account_id,
                                witness,
                                value,
                            )?
                        }
                        MatchingIdentifierWitness::Multi(account_id, witness) => {
                            self.multisig = input_multi_account_verify(
                                self.multisig,
                                &self.static_params.block0_initial_hash,
                                &sign_data_hash,
                                &account_id,
                                witness,
                                value,
                            )?
                        }
                    }
                }
            }
        }
        Ok(self)
    }

    fn apply_tx_outputs(
        mut self,
        fragment_id: FragmentId,
        outputs: OutputsSlice<'_>,
    ) -> Result<Self, Error> {
        let mut new_utxos = Vec::new();
        for (index, output) in outputs.iter().enumerate() {
            check::valid_output_value(&output)?;

            if output.address.discrimination() != self.static_params.discrimination {
                return Err(Error::InvalidDiscrimination);
            }
            match output.address.kind() {
                Kind::Single(_) => {
                    new_utxos.push((index as u8, output.clone()));
                }
                Kind::Group(_, account_id) => {
                    let account_id = account_id.clone().into();
                    // TODO: probably faster to just call add_account and check for already exists error
                    if !self.accounts.exists(&account_id) {
                        self.accounts =
                            self.accounts.add_account(&account_id, Value::zero(), ())?;
                    }
                    new_utxos.push((index as u8, output.clone()));
                }
                Kind::Account(identifier) => {
                    // don't have a way to make a newtype ref from the ref so .clone()
                    let account = identifier.clone().into();
                    self.add_value_or_create_account(&account, output.value)?;
                }
                Kind::Multisig(identifier) => {
                    let identifier = multisig::Identifier::from(*identifier);
                    self.multisig = self.multisig.add_value(&identifier, output.value)?;
                }
                Kind::Script(_identifier) => {
                    // TODO: scripts address kinds are not yet supported
                    return Err(Error::ScriptsAddressNotAllowedYet);
                }
            }
        }
        if !new_utxos.is_empty() {
            self.utxos = self.utxos.add(&fragment_id, &new_utxos)?;
        }
        Ok(self)
    }

    fn add_value_or_create_account(
        &mut self,
        account: &account::Identifier,
        value: Value,
    ) -> Result<(), Error> {
        self.accounts = match self.accounts.add_value(account, value) {
            Ok(accounts) => accounts,
            Err(account::LedgerError::NonExistent) => {
                self.accounts.add_account(account, value, ())?
            }
            Err(error) => return Err(error.into()),
        };
        Ok(())
    }

    fn apply_tx_fee(mut self, fee: Value) -> Result<Self, Error> {
        self.pots.append_fees(fee)?;
        Ok(self)
    }

    fn apply_input_to_utxo(
        mut self,
        sign_data_hash: &TransactionSignDataHash,
        utxo: &UtxoPointer,
        witness: &Witness,
    ) -> Result<Self, Error> {
        match witness {
            Witness::Account(_) => Err(Error::ExpectingUtxoWitness),
            Witness::Multisig(_) => Err(Error::ExpectingUtxoWitness),
            Witness::OldUtxo(pk, cc, signature) => {
                let (old_utxos, associated_output) = self
                    .oldutxos
                    .remove(&utxo.transaction_id, utxo.output_index)?;

                self.oldutxos = old_utxos;
                if utxo.value != associated_output.value {
                    return Err(Error::UtxoValueNotMatching {
                        expected: utxo.value,
                        value: associated_output.value,
                    });
                };

                if legacy::oldaddress_from_xpub(&associated_output.address, pk, cc)
                    == legacy::OldAddressMatchXPub::No
                {
                    return Err(Error::OldUtxoInvalidPublicKey {
                        utxo: *utxo,
                        output: associated_output,
                        witness: witness.clone(),
                    });
                };

                let data_to_verify = WitnessUtxoData::new(
                    &self.static_params.block0_initial_hash,
                    sign_data_hash,
                    WitnessUtxoVersion::Legacy,
                );
                let verified = signature.verify(&pk, &data_to_verify);
                if verified == chain_crypto::Verification::Failed {
                    return Err(Error::OldUtxoInvalidSignature {
                        utxo: *utxo,
                        output: associated_output,
                        witness: witness.clone(),
                    });
                };

                Ok(self)
            }
            Witness::Utxo(signature) => {
                let (new_utxos, associated_output) =
                    self.utxos.remove(&utxo.transaction_id, utxo.output_index)?;
                self.utxos = new_utxos;
                if utxo.value != associated_output.value {
                    return Err(Error::UtxoValueNotMatching {
                        expected: utxo.value,
                        value: associated_output.value,
                    });
                }

                let data_to_verify = WitnessUtxoData::new(
                    &self.static_params.block0_initial_hash,
                    sign_data_hash,
                    WitnessUtxoVersion::Normal,
                );
                let verified = signature.verify(
                    &associated_output.address.public_key().unwrap(),
                    &data_to_verify,
                );
                if verified == chain_crypto::Verification::Failed {
                    return Err(Error::UtxoInvalidSignature {
                        utxo: *utxo,
                        output: associated_output,
                        witness: witness.clone(),
                    });
                };
                Ok(self)
            }
        }
    }

    pub fn remaining_rewards(&self) -> Value {
        self.pots.rewards
    }

    pub fn treasury_value(&self) -> Value {
        self.pots.treasury.value()
    }
}

fn apply_old_declaration(
    fragment_id: &FragmentId,
    mut utxos: utxo::Ledger<legacy::OldAddress>,
    decl: &legacy::UtxoDeclaration,
) -> Result<utxo::Ledger<legacy::OldAddress>, Error> {
    assert!(decl.addrs.len() < 255);
    let mut outputs = Vec::with_capacity(decl.addrs.len());
    for (i, d) in decl.addrs.iter().enumerate() {
        let output = Output {
            address: d.0.clone(),
            value: d.1,
        };
        outputs.push((i as u8, output))
    }
    utxos = utxos.add(&fragment_id, &outputs)?;
    Ok(utxos)
}

fn calculate_fee<'a, Extra: Payload>(
    tx: &TransactionSlice<'a, Extra>,
    dyn_params: &LedgerParameters,
) -> Value {
    dyn_params.fees.calculate_tx(tx)
}

pub enum MatchingIdentifierWitness<'a> {
    Single(account::Identifier, &'a account::Witness),
    Multi(multisig::Identifier, &'a multisig::Witness),
}

fn match_identifier_witness<'a>(
    account: &UnspecifiedAccountIdentifier,
    witness: &'a Witness,
) -> Result<MatchingIdentifierWitness<'a>, Error> {
    match witness {
        Witness::OldUtxo(..) => Err(Error::ExpectingAccountWitness),
        Witness::Utxo(_) => Err(Error::ExpectingAccountWitness),
        Witness::Account(sig) => {
            // refine account to a single account identifier
            let account = account
                .to_single_account()
                .ok_or(Error::AccountIdentifierInvalid)?;
            Ok(MatchingIdentifierWitness::Single(account, sig))
        }
        Witness::Multisig(msignature) => {
            // refine account to a multisig account identifier
            let account = account.to_multi_account();
            Ok(MatchingIdentifierWitness::Multi(account, msignature))
        }
    }
}

fn input_single_account_verify<'a>(
    mut ledger: account::Ledger,
    block0_hash: &HeaderId,
    sign_data_hash: &TransactionSignDataHash,
    account: &account::Identifier,
    witness: &'a account::Witness,
    value: Value,
) -> Result<account::Ledger, Error> {
    // .remove_value() check if there's enough value and if not, returns a Err.
    let (new_ledger, spending_counter) = ledger.remove_value(&account, value)?;
    ledger = new_ledger;

    let tidsc = WitnessAccountData::new(block0_hash, sign_data_hash, spending_counter);
    let verified = witness.verify(&account.clone().into(), &tidsc);
    if verified == chain_crypto::Verification::Failed {
        return Err(Error::AccountInvalidSignature {
            account: account.clone(),
            witness: Witness::Account(witness.clone()),
        });
    };
    Ok(ledger)
}

fn input_multi_account_verify<'a>(
    mut ledger: multisig::Ledger,
    block0_hash: &HeaderId,
    sign_data_hash: &TransactionSignDataHash,
    account: &multisig::Identifier,
    witness: &'a multisig::Witness,
    value: Value,
) -> Result<multisig::Ledger, Error> {
    // .remove_value() check if there's enough value and if not, returns a Err.
    let (new_ledger, declaration, spending_counter) = ledger.remove_value(&account, value)?;

    let data_to_verify = WitnessMultisigData::new(&block0_hash, sign_data_hash, spending_counter);
    if !witness.verify(declaration, &data_to_verify) {
        return Err(Error::MultisigInvalidSignature {
            multisig: account.clone(),
            witness: Witness::Multisig(witness.clone()),
        });
    }
    ledger = new_ledger;
    Ok(ledger)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        account::{Identifier, SpendingCounter},
        accounting::account::account_state::AccountState,
        fee::LinearFee,
        key::Hash,
        multisig,
        //reward::RewardParams,
        setting::{FeesGoesTo, Settings},
        testing::{
            address::ArbitraryAddressDataValueVec,
            builders::{
                witness_builder::{make_witness, make_witnesses},
                TestTx, TestTxBuilder,
            },
            data::{AddressData, AddressDataValue},
            ledger::{ConfigBuilder, LedgerBuilder},
            verifiers::LedgerStateVerifier,
            TestGen,
        },
        transaction::Witness,
    };
    use chain_addr::Discrimination;
    use quickcheck::{Arbitrary, Gen, TestResult};
    use quickcheck_macros::quickcheck;
    use std::{fmt, iter};

    impl Arbitrary for LedgerStaticParameters {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            LedgerStaticParameters {
                block0_initial_hash: Arbitrary::arbitrary(g),
                block0_start_time: Arbitrary::arbitrary(g),
                discrimination: Arbitrary::arbitrary(g),
                kes_update_speed: Arbitrary::arbitrary(g),
            }
        }
    }

    impl Arbitrary for LedgerParameters {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            LedgerParameters {
                fees: Arbitrary::arbitrary(g),
                treasury_tax: Arbitrary::arbitrary(g),
                reward_params: Arbitrary::arbitrary(g),
                block_content_max_size: Arbitrary::arbitrary(g),
                epoch_stability_depth: Arbitrary::arbitrary(g),
                fees_goes_to: Arbitrary::arbitrary(g),
                committees: Arbitrary::arbitrary(g),
            }
        }
    }

    #[derive(Clone)]
    pub struct ArbitraryEmptyLedger(Ledger);

    impl Arbitrary for ArbitraryEmptyLedger {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut ledger = Ledger::empty(
                Arbitrary::arbitrary(g),
                Arbitrary::arbitrary(g),
                Arbitrary::arbitrary(g),
                Arbitrary::arbitrary(g),
            );

            ledger.date = Arbitrary::arbitrary(g);
            ledger.chain_length = Arbitrary::arbitrary(g);
            ArbitraryEmptyLedger(ledger)
        }
    }

    impl fmt::Debug for ArbitraryEmptyLedger {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Ledger")
                .field("chain_length", &self.0.chain_length())
                .field("settings", &self.0.settings)
                .field("date", &self.0.date())
                .field("era", &self.0.era())
                .field("static_params", &self.0.get_static_parameters().clone())
                .finish()
        }
    }

    impl Into<Ledger> for ArbitraryEmptyLedger {
        fn into(self) -> Ledger {
            self.0
        }
    }

    #[quickcheck]
    pub fn apply_empty_block_prop_test(
        mut context: HeaderContentEvalContext,
        ledger: ArbitraryEmptyLedger,
    ) -> TestResult {
        let ledger: Ledger = ledger.into();
        let should_succeed = context.chain_length == ledger.chain_length.increase()
            && context.block_date > ledger.date;

        let contents = Contents::empty();
        context.content_hash = contents.compute_hash();

        let result = ledger.apply_block(&ledger.get_ledger_parameters(), &contents, &context);
        match (result, should_succeed) {
            (Ok(_), true) => TestResult::passed(),
            (Ok(_), false) => TestResult::error("should pass"),
            (Err(err), true) => TestResult::error(format!("unexpected error: {}", err)),
            (Err(_), false) => TestResult::passed(),
        }
    }

    fn empty_transaction() -> TestTx {
        TestTx::new(
            TxBuilder::new()
                .set_payload(&NoExtra)
                .set_ios(&[], &[])
                .set_witnesses(&[])
                .set_payload_auth(&()),
        )
    }

    fn transaction_from_ios_only(inputs: &[Input], outputs: &[Output<Address>]) -> TestTx {
        TestTx::new(
            TxBuilder::new()
                .set_payload(&NoExtra)
                .set_ios(inputs, outputs)
                .set_witnesses(&[])
                .set_payload_auth(&()),
        )
    }

    fn single_transaction_sign_by(
        input: Input,
        block0_hash: &HeaderId,
        address_data: &AddressData,
    ) -> TestTx {
        let tx_builder = TxBuilder::new()
            .set_payload(&NoExtra)
            .set_ios(&[input], &[]);

        let witness = make_witness(
            &block0_hash,
            &address_data,
            &tx_builder.get_auth_data_for_witness().hash(),
        );
        let witnesses = vec![witness];

        TestTx::new(tx_builder.set_witnesses(&witnesses).set_payload_auth(&()))
    }

    #[quickcheck]
    pub fn match_identifier_witness_prop_test(
        id: UnspecifiedAccountIdentifier,
        witness: Witness,
    ) -> TestResult {
        let result = super::match_identifier_witness(&id, &witness);
        match (witness.clone(), result) {
            (Witness::OldUtxo(..), Ok(_)) => TestResult::error("expecting error, but got success"),
            (Witness::OldUtxo(..), Err(_)) => TestResult::passed(),
            (Witness::Utxo(_), Ok(_)) => TestResult::error("expecting error, but got success"),
            (Witness::Utxo(_), Err(_)) => TestResult::passed(),
            (Witness::Account(_), Ok(_)) => TestResult::passed(),
            (Witness::Account(_), Err(_)) => TestResult::error("unexpected error"),
            (Witness::Multisig(_), _) => TestResult::discard(),
        }
    }

    #[quickcheck]
    pub fn input_single_account_verify_negative_prop_test(
        id: Identifier,
        account_state: AccountState<()>,
        value_to_sub: Value,
        block0_hash: HeaderId,
        sign_data_hash: TransactionSignDataHash,
        witness: account::Witness,
    ) -> TestResult {
        let mut account_ledger = account::Ledger::new();
        account_ledger = account_ledger
            .add_account(&id, account_state.get_value(), ())
            .unwrap();
        let result = super::input_single_account_verify(
            account_ledger,
            &block0_hash,
            &sign_data_hash,
            &id,
            &witness,
            value_to_sub,
        );

        TestResult::from_bool(result.is_err())
    }

    #[test]
    pub fn test_input_single_account_verify_correct_account() {
        let account = AddressData::account(Discrimination::Test);
        let initial_value = Value(100);
        let value_to_sub = Value(80);
        let block0_hash = TestGen::hash();
        let id: Identifier = account.public_key().into();

        let account_ledger = account_ledger_with_initials(&[(id.clone(), initial_value)]);
        let signed_tx = single_transaction_sign_by(
            account.make_input(initial_value, None),
            &block0_hash,
            &account,
        );
        let sign_data_hash = signed_tx.hash();

        let result = super::input_single_account_verify(
            account_ledger,
            &block0_hash,
            &sign_data_hash,
            &id,
            &to_account_witness(&signed_tx.witnesses().iter().next().unwrap()),
            value_to_sub,
        );
        assert!(result.is_ok())
    }

    fn account_ledger_with_initials(initials: &[(Identifier, Value)]) -> account::Ledger {
        let mut account_ledger = account::Ledger::new();
        for (id, initial_value) in initials {
            account_ledger = account_ledger.add_account(&id, *initial_value, ()).unwrap();
        }
        account_ledger
    }

    #[test]
    pub fn test_input_single_account_verify_different_block0_hash() {
        let account = AddressData::account(Discrimination::Test);
        let initial_value = Value(100);
        let value_to_sub = Value(80);
        let block0_hash = TestGen::hash();
        let wrong_block0_hash = TestGen::hash();
        let id: Identifier = account.public_key().into();

        let account_ledger = account_ledger_with_initials(&[(id.clone(), initial_value)]);
        let signed_tx = single_transaction_sign_by(
            account.make_input(initial_value, None),
            &block0_hash,
            &account,
        );
        let sign_data_hash = signed_tx.hash();

        let result = super::input_single_account_verify(
            account_ledger,
            &wrong_block0_hash,
            &sign_data_hash,
            &id,
            &to_account_witness(&signed_tx.witnesses().iter().next().unwrap()),
            value_to_sub,
        );
        assert!(result.is_err())
    }

    fn to_account_witness(witness: &Witness) -> &account::Witness {
        match witness {
            Witness::Account(account_witness) => account_witness,
            _ => panic!("wrong type of witness"),
        }
    }

    #[test]
    pub fn test_input_account_wrong_value() {
        let account = AddressData::account(Discrimination::Test);
        let initial_value = Value(100);
        let value_to_sub = Value(110);
        let block0_hash = TestGen::hash();
        let wrong_block0_hash = TestGen::hash();
        let id: Identifier = account.public_key().into();

        let account_ledger = account_ledger_with_initials(&[(id.clone(), initial_value)]);
        let signed_tx = single_transaction_sign_by(
            account.make_input(initial_value, None),
            &block0_hash,
            &account,
        );
        let sign_data_hash = signed_tx.hash();

        let result = super::input_single_account_verify(
            account_ledger,
            &wrong_block0_hash,
            &sign_data_hash,
            &id,
            &to_account_witness(&signed_tx.witnesses().iter().next().unwrap()),
            value_to_sub,
        );
        assert!(result.is_err())
    }

    #[test]
    pub fn test_input_single_account_verify_non_existing_account() {
        let account = AddressData::account(Discrimination::Test);
        let non_existing_account = AddressData::account(Discrimination::Test);
        let initial_value = Value(100);
        let value_to_sub = Value(110);
        let block0_hash = TestGen::hash();
        let wrong_block0_hash = TestGen::hash();
        let id: Identifier = account.public_key().into();

        let account_ledger = account_ledger_with_initials(&[(id, initial_value)]);
        let signed_tx = single_transaction_sign_by(
            account.make_input(initial_value, None),
            &block0_hash,
            &account,
        );
        let sign_data_hash = signed_tx.hash();

        let result = super::input_single_account_verify(
            account_ledger,
            &wrong_block0_hash,
            &sign_data_hash,
            &non_existing_account.public_key().into(),
            &to_account_witness(&signed_tx.witnesses().iter().next().unwrap()),
            value_to_sub,
        );
        assert!(result.is_err())
    }

    #[quickcheck]
    pub fn input_utxo_verify_negative_prop_test(
        sign_data_hash: TransactionSignDataHash,
        utxo_pointer: UtxoPointer,
        witness: Witness,
    ) -> TestResult {
        let faucet = AddressDataValue::utxo(Discrimination::Test, Value(1000));
        let test_ledger = LedgerBuilder::from_config(ConfigBuilder::new(0))
            .faucet(&faucet)
            .build()
            .unwrap();

        let inner_ledger: Ledger = test_ledger.into();
        let result = inner_ledger.apply_input_to_utxo(&sign_data_hash, &utxo_pointer, &witness);
        match (witness, result) {
            (Witness::OldUtxo(..), Ok(_)) => TestResult::error("expecting error, but got success"),
            (Witness::OldUtxo(..), Err(_)) => TestResult::passed(),
            (Witness::Utxo(_), Ok(_)) => TestResult::error("expecting error, but got success"),
            (Witness::Utxo(_), Err(_)) => TestResult::passed(),
            (Witness::Account(_), Ok(_)) => TestResult::error("expecting error, but got success"),
            (Witness::Account(_), Err(_)) => TestResult::passed(),
            (Witness::Multisig(_), _) => TestResult::discard(),
        }
    }

    #[test]
    pub fn test_input_utxo_verify_correct_utxo() {
        let faucet = AddressDataValue::utxo(Discrimination::Test, Value(1000));
        let test_ledger = LedgerBuilder::from_config(ConfigBuilder::new(0))
            .faucet(&faucet)
            .build()
            .unwrap();

        let block0_hash = test_ledger.block0_hash;
        let ledger: Ledger = test_ledger.into();

        let utxo = ledger.utxos().next().unwrap();
        let utxo_pointer = UtxoPointer::new(utxo.fragment_id, utxo.output_index, utxo.output.value);

        let signed_tx =
            single_transaction_sign_by(faucet.make_input(Some(utxo)), &block0_hash, &faucet.into());
        let sign_data_hash = signed_tx.hash();
        let result = ledger.apply_input_to_utxo(
            &sign_data_hash,
            &utxo_pointer,
            &signed_tx.witnesses().iter().next().unwrap(),
        );
        assert!(result.is_ok())
    }

    #[test]
    pub fn test_input_utxo_verify_incorrect_value() {
        let faucet = AddressDataValue::utxo(Discrimination::Test, Value(1000));
        let test_ledger = LedgerBuilder::from_config(ConfigBuilder::new(0))
            .faucet(&faucet)
            .build()
            .unwrap();

        let block0_hash = test_ledger.block0_hash;
        let ledger: Ledger = test_ledger.into();

        let utxo = ledger.utxos().next().unwrap();
        let utxo_pointer = UtxoPointer::new(utxo.fragment_id, utxo.output_index, Value(10));
        let signed_tx =
            single_transaction_sign_by(faucet.make_input(Some(utxo)), &block0_hash, &faucet.into());
        let sign_data_hash = signed_tx.hash();
        let result = ledger.apply_input_to_utxo(
            &sign_data_hash,
            &utxo_pointer,
            &signed_tx.witnesses().iter().next().unwrap(),
        );
        assert!(result.is_err())
    }

    #[quickcheck]
    pub fn test_internal_apply_transaction_output_property(
        utxos: utxo::Ledger<Address>,
        accounts: account::Ledger,
        static_params: LedgerStaticParameters,
        transaction_id: FragmentId,
        arbitrary_outputs: ArbitraryAddressDataValueVec,
    ) -> TestResult {
        let multisig_ledger = multisig::Ledger::new();
        let outputs: Vec<Output<Address>> = arbitrary_outputs
            .0
            .iter()
            .map(|x| x.address_data.make_output(x.value))
            .collect();

        let ledger = build_ledger(utxos, accounts, multisig_ledger, static_params.clone());
        let auth_tx = transaction_from_ios_only(&[], &outputs);
        let result = ledger.apply_tx_outputs(transaction_id, auth_tx.get_tx_outputs());

        match (
            should_expect_success(arbitrary_outputs, &static_params),
            result,
        ) {
            (true, Ok(_)) => TestResult::passed(),
            (true, Err(err)) => TestResult::error(format!("Unexpected failure: {:?}", err)),
            (false, Ok(_)) => TestResult::error("Expected failure, but got sucess"),
            (false, Err(_)) => TestResult::passed(),
        }
    }

    fn build_ledger(
        utxos: utxo::Ledger<Address>,
        accounts: account::Ledger,
        multisig_ledger: multisig::Ledger,
        static_params: LedgerStaticParameters,
    ) -> Ledger {
        let mut ledger = Ledger::empty(
            Settings::new(),
            static_params,
            build_time_era(),
            Pots::zero(),
        );

        ledger.utxos = utxos;
        ledger.accounts = accounts;
        ledger.multisig = multisig_ledger;
        ledger
    }

    fn build_time_era() -> TimeEra {
        let now = SystemTime::now();
        let t0 = Timeline::new(now);
        let f0 = SlotDuration::from_secs(5);
        let tf0 = TimeFrame::new(t0, f0);
        let t1 = now + Duration::from_secs(10);
        let slot1 = tf0.slot_at(&t1).unwrap();
        TimeEra::new(slot1, TimeEpoch(2), 4)
    }

    fn should_expect_success(
        arbitrary_outputs: ArbitraryAddressDataValueVec,
        static_params: &LedgerStaticParameters,
    ) -> bool {
        let is_any_address_different_than_ledger_disc = arbitrary_outputs
            .0
            .iter()
            .any(|x| x.address_data.discrimination() != static_params.discrimination);
        let is_any_address_zero_output =
            arbitrary_outputs.0.iter().any(|x| x.value == Value::zero());
        !is_any_address_different_than_ledger_disc && !is_any_address_zero_output
    }

    #[derive(Clone, Debug)]
    pub struct InternalApplyTransactionTestParams {
        pub dyn_params: LedgerParameters,
        pub static_params: LedgerStaticParameters,
        pub transaction_id: Hash,
    }

    impl InternalApplyTransactionTestParams {
        pub fn new() -> Self {
            InternalApplyTransactionTestParams::new_with_fee(LinearFee::new(0, 0, 0))
        }

        pub fn new_with_fee(fees: LinearFee) -> Self {
            let static_params = LedgerStaticParameters {
                block0_initial_hash: TestGen::hash(),
                block0_start_time: config::Block0Date(0),
                discrimination: Discrimination::Test,
                kes_update_speed: 100,
            };

            let dyn_params = LedgerParameters {
                fees,
                treasury_tax: rewards::TaxType::zero(),
                reward_params: rewards::Parameters::zero(),
                block_content_max_size: 10_240,
                epoch_stability_depth: 1000,
                fees_goes_to: FeesGoesTo::Rewards,
                committees: Arc::new(Vec::new()),
            };
            InternalApplyTransactionTestParams {
                dyn_params,
                static_params,
                transaction_id: TestGen::hash(),
            }
        }

        pub fn transaction_id(&self) -> Hash {
            self.transaction_id
        }

        pub fn static_params(&self) -> LedgerStaticParameters {
            self.static_params.clone()
        }

        pub fn expected_account_with_value(&self, value: Value) -> AccountState<()> {
            AccountState::new(value, ())
        }

        pub fn expected_utxo_entry<'a>(
            &self,
            output: &'a OutputAddress,
        ) -> utxo::Entry<'a, Address> {
            utxo::Entry {
                fragment_id: self.transaction_id(),
                output_index: 0 as u8,
                output,
            }
        }
    }

    #[test]
    pub fn test_internal_apply_transaction_output_delegation_for_existing_account() {
        let params = InternalApplyTransactionTestParams::new();

        let multisig_ledger = multisig::Ledger::new();
        let utxos = utxo::Ledger::new();
        let mut accounts = account::Ledger::new();

        let account = AddressData::account(Discrimination::Test);
        accounts = accounts
            .add_account(&account.to_id(), Value(100), ())
            .unwrap();

        let delegation = AddressData::delegation_for(&account);
        let delegation_output = delegation.make_output(Value(100));

        let ledger = build_ledger(utxos, accounts, multisig_ledger, params.static_params());
        let auth_tx = transaction_from_ios_only(
            &[],
            &[delegation_output.clone(), account.make_output(Value(1))],
        );

        let ledger = ledger
            .apply_tx_outputs(params.transaction_id(), auth_tx.get_tx_outputs())
            .expect("Unexpected error while applying transaction output");

        LedgerStateVerifier::new(ledger)
            .utxos_count_is(1)
            .and()
            .accounts_count_is(1)
            .and()
            .multisigs_count_is_zero()
            .and()
            .utxo_contains(&params.expected_utxo_entry(&delegation_output))
            .and()
            .accounts_contains(
                account.to_id(),
                params.expected_account_with_value(Value(101)),
            );
    }

    #[test]
    pub fn test_internal_apply_transaction_output_delegation_non_existing_account() {
        let params = InternalApplyTransactionTestParams::new();

        let multisig_ledger = multisig::Ledger::new();
        let utxos = utxo::Ledger::new();
        let accounts = account::Ledger::new();

        let delegation_address = AddressData::delegation(Discrimination::Test);
        let delegation_output = delegation_address.make_output(Value(100));

        let ledger = build_ledger(utxos, accounts, multisig_ledger, params.static_params());

        let auth_tx = transaction_from_ios_only(&[], &[delegation_output.clone()]);
        let ledger = ledger
            .apply_tx_outputs(params.transaction_id(), auth_tx.get_tx_outputs())
            .expect("Unexpected error while applying transaction output");

        LedgerStateVerifier::new(ledger)
            .utxos_count_is(1)
            .and()
            .accounts_count_is(1)
            .and()
            .multisigs_count_is_zero()
            .and()
            .utxo_contains(&params.expected_utxo_entry(&delegation_output))
            .and()
            .accounts_contains(
                delegation_address.delegation_id(),
                params.expected_account_with_value(Value(0)),
            );
    }

    #[test]
    pub fn test_internal_apply_transaction_output_account_for_existing_account() {
        let params = InternalApplyTransactionTestParams::new();

        let multisig_ledger = multisig::Ledger::new();
        let utxos = utxo::Ledger::new();
        let mut accounts = account::Ledger::new();

        let account = AddressData::account(Discrimination::Test);
        accounts = accounts
            .add_account(&account.to_id(), Value(100), ())
            .unwrap();

        let ledger = build_ledger(utxos, accounts, multisig_ledger, params.static_params());

        let auth_tx = transaction_from_ios_only(&[], &[account.make_output(Value(200))]);
        let ledger = ledger
            .apply_tx_outputs(params.transaction_id(), auth_tx.get_tx_outputs())
            .expect("Unexpected error while applying transaction output");

        LedgerStateVerifier::new(ledger)
            .utxos_count_is(0)
            .and()
            .accounts_count_is(1)
            .and()
            .multisigs_count_is_zero()
            .and()
            .accounts_contains(
                account.to_id(),
                params.expected_account_with_value(Value(300)),
            );
    }

    #[test]
    pub fn test_internal_apply_transaction_output_account_for_non_existing_account() {
        let params = InternalApplyTransactionTestParams::new();

        let multisig_ledger = multisig::Ledger::new();
        let utxos = utxo::Ledger::new();
        let accounts = account::Ledger::new();
        let account = AddressData::account(Discrimination::Test);

        let ledger = build_ledger(utxos, accounts, multisig_ledger, params.static_params());
        let auth_tx = transaction_from_ios_only(&[], &[account.make_output(Value(200))]);
        let ledger = ledger
            .apply_tx_outputs(params.transaction_id(), auth_tx.get_tx_outputs())
            .expect("Unexpected error while applying transaction output");

        LedgerStateVerifier::new(ledger)
            .utxos_count_is(0)
            .and()
            .accounts_count_is(1)
            .and()
            .multisigs_count_is_zero()
            .and()
            .accounts_contains(
                account.to_id(),
                params.expected_account_with_value(Value(200)),
            );
    }

    #[test]
    pub fn test_internal_apply_transaction_output_empty() {
        let params = InternalApplyTransactionTestParams::new();

        let multisig_ledger = multisig::Ledger::new();
        let utxos = utxo::Ledger::new();
        let accounts = account::Ledger::new();

        let ledger = build_ledger(utxos, accounts, multisig_ledger, params.static_params());

        let auth_tx = empty_transaction();

        let ledger = ledger
            .apply_tx_outputs(params.transaction_id(), auth_tx.get_tx_outputs())
            .expect("Unexpected error while applying transaction output");

        LedgerStateVerifier::new(ledger)
            .utxos_count_is(0)
            .and()
            .accounts_count_is(0)
            .and()
            .multisigs_count_is_zero();
    }

    /// internal_apply_transaction
    #[test]
    pub fn test_internal_apply_transaction_max_witnesses() {
        let faucets: Vec<AddressDataValue> =
            iter::from_fn(|| Some(AddressDataValue::account(Discrimination::Test, Value(1))))
                .take(check::CHECK_TX_MAXIMUM_INPUTS as usize)
                .collect();
        let reciever = AddressData::utxo(Discrimination::Test);

        let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new(0))
            .faucets(&faucets)
            .build()
            .unwrap();

        let inputs: Vec<Input> = faucets.iter().map(|x| x.make_input(None)).collect();

        let builder_tx = TxBuilder::new()
            .set_payload(&NoExtra)
            .set_ios(&inputs, &[reciever.make_output(Value(100))]);

        let witnesses: Vec<Witness> = faucets
            .iter()
            .map(|faucet| {
                make_witness(
                    &test_ledger.block0_hash,
                    &faucet.clone().into(),
                    &builder_tx.get_auth_data_for_witness().hash(),
                )
            })
            .take(check::CHECK_TX_MAXIMUM_INPUTS as usize)
            .collect();

        let tx = builder_tx.set_witnesses(&witnesses).set_payload_auth(&());

        let fragment = TestTx::new(tx).get_fragment();
        assert!(test_ledger.apply_transaction(fragment).is_err());
    }

    #[test]
    pub fn test_internal_apply_transaction_max_outputs() {
        let faucet = AddressDataValue::utxo(
            Discrimination::Test,
            Value(check::CHECK_TX_MAXIMUM_INPUTS.into()),
        );
        let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new(0))
            .faucet(&faucet)
            .build()
            .unwrap();

        let receivers =
            iter::from_fn(|| Some(AddressDataValue::account(Discrimination::Test, Value(100))))
                .take(check::CHECK_TX_MAXIMUM_INPUTS as usize)
                .collect::<Vec<_>>();

        let test_tx = TestTxBuilder::new(test_ledger.block0_hash).move_funds_multiple(
            &mut test_ledger,
            &[faucet],
            &receivers,
        );
        println!(
            "{:?}",
            test_ledger.apply_transaction(test_tx.get_fragment())
        );
        TestResult::error("");
    }

    #[test]
    pub fn test_internal_apply_transaction_max_inputs() {
        let faucets: Vec<AddressDataValue> =
            iter::from_fn(|| Some(AddressDataValue::account(Discrimination::Test, Value(1))))
                .take(check::CHECK_TX_MAXIMUM_INPUTS as usize)
                .collect();

        let receiver = AddressDataValue::utxo(
            Discrimination::Test,
            Value((check::CHECK_TX_MAXIMUM_INPUTS as u64) + 1),
        );
        let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new(0))
            .faucets(&faucets)
            .build()
            .unwrap();

        let test_tx = TestTxBuilder::new(test_ledger.block0_hash).move_funds_multiple(
            &mut test_ledger,
            &faucets,
            &[receiver],
        );
        assert!(test_ledger
            .apply_transaction(test_tx.get_fragment())
            .is_err());
    }

    #[test]
    pub fn test_internal_apply_transaction_same_witness_for_all_input() {
        let faucets = vec![
            AddressDataValue::account(Discrimination::Test, Value(1)),
            AddressDataValue::account(Discrimination::Test, Value(1)),
        ];
        let reciever = AddressData::utxo(Discrimination::Test);
        let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new(0))
            .faucets(&faucets)
            .build()
            .unwrap();

        let inputs: Vec<Input> = faucets.iter().map(|x| x.make_input(None)).collect();
        let tx_builder = TxBuilder::new()
            .set_payload(&NoExtra)
            .set_ios(&inputs, &[reciever.make_output(Value(2))]);

        let witness = make_witness(
            &test_ledger.block0_hash,
            &faucets[0].clone().into(),
            &tx_builder.get_auth_data_for_witness().hash(),
        );
        let test_tx = TestTx::new(
            tx_builder
                .set_witnesses(&[witness.clone(), witness])
                .set_payload_auth(&()),
        );

        assert!(test_ledger
            .apply_transaction(test_tx.get_fragment())
            .is_err());
    }

    #[test]
    pub fn test_internal_apply_transaction_verify_pot() {
        let faucet = AddressDataValue::account(Discrimination::Test, Value(101));
        let reciever = AddressDataValue::account(Discrimination::Test, Value(1));

        let mut test_ledger =
            LedgerBuilder::from_config(ConfigBuilder::new(0).with_fee(LinearFee::new(10, 1, 1)))
                .faucet(&faucet)
                .build()
                .unwrap();

        let test_tx = TestTxBuilder::new(test_ledger.block0_hash).move_all_funds(
            &mut test_ledger,
            &faucet,
            &reciever,
        );
        assert!(test_ledger
            .apply_transaction(test_tx.get_fragment())
            .is_ok());
        LedgerStateVerifier::new(test_ledger.into())
            .pots()
            .has_fee_equals_to(&Value(12));
    }

    #[quickcheck]
    pub fn test_internal_apply_transaction_is_balanced(
        input_addresses: ArbitraryAddressDataValueVec,
        output_addresses: ArbitraryAddressDataValueVec,
        fee: Value,
    ) -> TestResult {
        if input_addresses.is_empty() || output_addresses.is_empty() {
            return TestResult::discard();
        }

        let mut test_ledger =
            LedgerBuilder::from_config(ConfigBuilder::new(0).with_fee(LinearFee::new(fee.0, 0, 0)))
                .faucets(&input_addresses.values())
                .build()
                .unwrap();

        let block0_hash = test_ledger.block0_hash;
        let tx_builder = TxBuilder::new().set_payload(&NoExtra).set_ios(
            &input_addresses.make_inputs(&test_ledger),
            &output_addresses.make_outputs(),
        );

        let witnesses: Vec<Witness> = input_addresses
            .as_addresses()
            .iter()
            .map(|x| {
                make_witness(
                    &block0_hash,
                    x,
                    &tx_builder.get_auth_data_for_witness().hash(),
                )
            })
            .collect();

        let test_tx = TestTx::new(tx_builder.set_witnesses(&witnesses).set_payload_auth(&()));

        let balance_res = (input_addresses.total_value() - output_addresses.total_value())
            .and_then(|balance| balance - fee);
        match (
            balance_res,
            test_ledger.apply_transaction(test_tx.get_fragment()),
        ) {
            (Ok(balance), Ok(_)) => TestResult::from_bool(balance == Value::zero()),
            (Err(err), Ok(_)) => TestResult::error(format!(
                "Expected balance is non zero {:?}, yet transaction is accepted",
                err
            )),
            (Ok(balance), Err(_)) => TestResult::from_bool(balance != Value::zero()),
            (Err(_), Err(_)) => TestResult::passed(),
        }
    }

    #[test]
    pub fn test_internal_apply_transaction_witness_collection_should_be_ordered_as_inputs() {
        let faucets = vec![
            AddressDataValue::account(Discrimination::Test, Value(1)),
            AddressDataValue::account(Discrimination::Test, Value(1)),
        ];
        let reciever = AddressData::utxo(Discrimination::Test);
        let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new(0))
            .faucets(&faucets)
            .build()
            .unwrap();

        let inputs = [faucets[0].make_input(None), faucets[1].make_input(None)];
        let tx_builder = TxBuilder::new()
            .set_payload(&NoExtra)
            .set_ios(&inputs, &[reciever.make_output(Value(2))]);
        let auth_data = tx_builder.get_auth_data_for_witness().hash();
        let witnesses = make_witnesses(
            &test_ledger.block0_hash,
            vec![&faucets[1].clone().into(), &faucets[0].clone().into()],
            &auth_data,
        );

        let tx = tx_builder.set_witnesses(&witnesses).set_payload_auth(&());
        let test_tx = TestTx::new(tx);
        assert!(test_ledger
            .apply_transaction(test_tx.get_fragment())
            .is_err());
    }

    #[test]
    pub fn test_internal_apply_transaction_no_inputs_outputs() {
        let faucet = AddressDataValue::account(Discrimination::Test, Value(1));
        let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new(0))
            .faucet(&faucet)
            .build()
            .unwrap();

        let test_tx = single_transaction_sign_by(
            faucet.make_input(None),
            &test_ledger.block0_hash,
            &faucet.into(),
        );

        assert!(test_ledger
            .apply_transaction(test_tx.get_fragment())
            .is_err());
    }

    #[quickcheck]
    pub fn test_internal_apply_transaction_funds_were_transfered(
        sender_address: AddressData,
        reciever_address: AddressData,
    ) {
        let config_builder = ConfigBuilder::new(0)
            .with_rewards(Value(0))
            .with_treasury(Value(0));

        let faucet = AddressDataValue::new(sender_address, Value(1));
        let reciever = AddressDataValue::new(reciever_address, Value(1));
        let mut test_ledger = LedgerBuilder::from_config(config_builder)
            .faucet(&faucet)
            .build()
            .unwrap();

        let fragment = TestTxBuilder::new(test_ledger.block0_hash)
            .move_all_funds(&mut test_ledger, &faucet, &reciever)
            .get_fragment();
        assert!(test_ledger.apply_transaction(fragment).is_ok());

        LedgerStateVerifier::new(test_ledger.into())
            .address_has_expected_balance(reciever.into(), Value(1))
            .and()
            .address_has_expected_balance(faucet.into(), Value(0))
            .and()
            .total_value_is(&Value(1));
    }

    #[test]
    pub fn test_internal_apply_transaction_wrong_witness_type() {
        let faucet = AddressDataValue::utxo(Discrimination::Test, Value(1));
        let reciever = AddressDataValue::account(Discrimination::Test, Value(1));
        let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new(0))
            .faucet(&faucet)
            .build()
            .unwrap();

        let utxo = test_ledger.utxos().next();

        let tx_builder = TxBuilder::new()
            .set_payload(&NoExtra)
            .set_ios(&[faucet.make_input(utxo)], &[reciever.make_output()]);

        let witness = Witness::new_account(
            &test_ledger.block0_hash,
            &tx_builder.get_auth_data_for_witness().hash(),
            SpendingCounter::zero(),
            |d| faucet.private_key().sign(d),
        );

        let tx = tx_builder.set_witnesses(&[witness]).set_payload_auth(&());
        let test_tx = TestTx::new(tx);

        assert!(test_ledger
            .apply_transaction(test_tx.get_fragment())
            .is_err());
    }

    #[test]
    pub fn test_internal_apply_transaction_wrong_transaction_hash() {
        let faucet = AddressDataValue::account(Discrimination::Test, Value(1));
        let reciever = AddressDataValue::account(Discrimination::Test, Value(1));
        let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new(0))
            .faucet(&faucet)
            .build()
            .unwrap();

        let tx_builder = TxBuilder::new().set_payload(&NoExtra);
        let tx_builder = tx_builder.set_ios(&[faucet.make_input(None)], &[reciever.make_output()]);

        let random_bytes = TestGen::bytes();
        let auth_data = TransactionAuthData(&random_bytes);

        let witness = make_witness(&test_ledger.block0_hash, &faucet.into(), &auth_data.hash());

        let tx = tx_builder.set_witnesses(&[witness]).set_payload_auth(&());
        let test_tx = TestTx::new(tx);
        assert!(test_ledger
            .apply_transaction(test_tx.get_fragment())
            .is_err());
    }

    #[test]
    pub fn test_internal_apply_transaction_wrong_block0_hash() {
        let wrong_block0_hash = TestGen::hash();
        let faucet = AddressDataValue::account(Discrimination::Test, Value(1));
        let reciever = AddressDataValue::account(Discrimination::Test, Value(1));

        let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new(0))
            .faucet(&faucet)
            .build()
            .unwrap();

        let tx_builder = TxBuilder::new()
            .set_payload(&NoExtra)
            .set_ios(&[faucet.make_input(None)], &[reciever.make_output()]);

        let witness = make_witness(
            &wrong_block0_hash,
            &faucet.into(),
            &tx_builder.get_auth_data_for_witness().hash(),
        );

        let tx = tx_builder.set_witnesses(&[witness]).set_payload_auth(&());
        let test_tx = TestTx::new(tx);

        assert!(test_ledger
            .apply_transaction(test_tx.get_fragment())
            .is_err());
    }

    #[test]
    pub fn test_internal_apply_transaction_wrong_spending_counter() {
        let faucet =
            AddressDataValue::account_with_spending_counter(Discrimination::Test, 1, Value(1));
        let reciever = AddressDataValue::account(Discrimination::Test, Value(1));

        let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new(0))
            .faucet(&faucet)
            .build()
            .unwrap();

        let tx_builder = TxBuilder::new()
            .set_payload(&NoExtra)
            .set_ios(&[faucet.make_input(None)], &[reciever.make_output()]);

        let witness = make_witness(
            &test_ledger.block0_hash,
            &faucet.into(),
            &tx_builder.get_auth_data_for_witness().hash(),
        );

        let tx = tx_builder.set_witnesses(&[witness]).set_payload_auth(&());
        let test_tx = TestTx::new(tx);

        assert!(test_ledger
            .apply_transaction(test_tx.get_fragment())
            .is_err());
    }

    #[test]
    pub fn test_internal_apply_transaction_wrong_private_key() {
        let faucet = AddressDataValue::account(Discrimination::Test, Value(1));
        let reciever = AddressDataValue::account(Discrimination::Test, Value(1));

        let mut test_ledger = LedgerBuilder::from_config(ConfigBuilder::new(0))
            .faucet(&faucet)
            .build()
            .unwrap();

        let tx_builder = TxBuilder::new()
            .set_payload(&NoExtra)
            .set_ios(&[faucet.make_input(None)], &[reciever.make_output()]);

        let witness = make_witness(
            &test_ledger.block0_hash,
            &reciever.into(),
            &tx_builder.get_auth_data_for_witness().hash(),
        );
        let tx = tx_builder.set_witnesses(&[witness]).set_payload_auth(&());
        let test_tx = TestTx::new(tx);
        assert!(test_ledger
            .apply_transaction(test_tx.get_fragment())
            .is_err());
    }
}
