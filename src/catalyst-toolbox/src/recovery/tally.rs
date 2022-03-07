use chain_addr::{Discrimination, Kind};
use chain_core::property::Fragment as _;
use chain_crypto::{Ed25519Extended, SecretKey};
use chain_impl_mockchain::{
    account::{self, LedgerError, SpendingCounter},
    block::{Block, BlockDate, HeaderId},
    certificate::{self, VoteCast, VotePlan, VotePlanId},
    chaineval::ConsensusEvalContext,
    fee::{FeeAlgorithm, LinearFee},
    fragment::{Fragment, FragmentId},
    ledger::{self, Ledger},
    transaction::{
        InputEnum, NoExtra, Output, TransactionSignDataHash, TransactionSlice, Witness,
        WitnessAccountData,
    },
    value::ValueError,
    vote::CommitteeId,
};
use chain_time::{Epoch, Slot, SlotDuration, TimeEra, TimeFrame, Timeline};
use jormungandr_lib::{
    crypto::{account::Identifier, hash::Hash},
    interfaces::{
        Address, Block0Configuration, CommitteeIdDef, FragmentLogDeserializeError, Initial,
        InitialUTxO, PersistentFragmentLog, SlotDuration as Block0SlotDuration,
    },
    time::SecondsSinceUnixEpoch,
};
use log::{debug, error, trace, warn};
use std::collections::{HashMap, HashSet};
use std::ops::{Add, Range};
use std::time::{Duration, SystemTime};
use wallet::{Settings, TransactionBuilder, Wallet};

#[allow(clippy::large_enum_variant)]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DeserializeError(#[from] jormungandr_lib::interfaces::FragmentLogDeserializeError),

    #[error(transparent)]
    LedgerError(#[from] chain_impl_mockchain::ledger::Error),

    #[error(transparent)]
    Block0ConfigurationError(#[from] jormungandr_lib::interfaces::Block0ConfigurationError),

    #[error(transparent)]
    ValidationError(#[from] ValidationError),

    #[error(transparent)]
    ReplayError(#[from] ReplayError),
}

fn timestamp_to_system_time(ts: SecondsSinceUnixEpoch) -> SystemTime {
    SystemTime::UNIX_EPOCH.add(Duration::new(ts.to_secs(), 0))
}

fn fragment_log_timestamp_to_blockdate(
    timestamp: SecondsSinceUnixEpoch,
    timeframe: &TimeFrame,
    era: &TimeEra,
) -> Option<BlockDate> {
    let slot = timestamp_to_system_time(timestamp);
    let new_slot = timeframe.slot_at(&slot)?;
    let epoch_position = era.from_slot_to_era(new_slot)?;
    Some(BlockDate::from(epoch_position))
}

fn timeframe_from_block0_start_and_slot_duration(
    block0_start: SecondsSinceUnixEpoch,
    slot_duration: Block0SlotDuration,
) -> TimeFrame {
    let timeline = Timeline::new(timestamp_to_system_time(block0_start));
    TimeFrame::new(
        timeline,
        SlotDuration::from_secs(<u8>::from(slot_duration) as u32),
    )
}

fn committee_id_to_address(id: CommitteeIdDef) -> Address {
    let id = CommitteeId::from(id);
    let pk = id.public_key();
    chain_addr::Address(Discrimination::Production, Kind::Account(pk)).into()
}

fn voteplans_from_block0(block0: &Block) -> HashMap<VotePlanId, VotePlan> {
    block0
        .fragments()
        .filter_map(|fragment| {
            if let Fragment::VotePlan(tx) = fragment {
                let voteplan = tx.as_slice().payload().into_payload();
                Some((voteplan.to_id(), voteplan))
            } else {
                None
            }
        })
        .collect()
}

/// check that the transaction input/outputs/witnesses is valid for the ballot
/// * Only 1 input (subsequently 1 witness), no output
pub(crate) fn valid_vote_cast(tx: &TransactionSlice<certificate::VoteCast>) -> bool {
    !(tx.inputs().nb_inputs() != 1
        || tx.witnesses().nb_witnesses() != 1
        || tx.outputs().nb_outputs() != 0)
}

fn verify_original_tx(
    spending_counter: SpendingCounter,
    block0_hash: &HeaderId,
    sign_data_hash: &TransactionSignDataHash,
    account: &account::Identifier,
    witness: &account::Witness,
    range_check: Range<u32>,
) -> (bool, SpendingCounter) {
    let spending_counter: u32 = <u32>::from(spending_counter);
    for i in range_check {
        for op in &[u32::checked_add, u32::checked_sub] {
            if let Some(new_spending_counter) = op(spending_counter, i) {
                let tidsc = WitnessAccountData::new(
                    block0_hash,
                    sign_data_hash,
                    SpendingCounter::from(new_spending_counter),
                );
                if witness.verify(account.as_ref(), &tidsc) == chain_crypto::Verification::Success {
                    trace!(
                        "expected: {} found: {}",
                        spending_counter,
                        new_spending_counter
                    );
                    return (true, new_spending_counter.into());
                }
            }
        }
    }
    (false, 0.into())
}

fn increment_ledger_time_up_to(ledger: &Ledger, blockdate: BlockDate) -> Ledger {
    ledger
        .begin_block(ledger.chain_length().increase(), blockdate)
        .unwrap()
        .finish(&ConsensusEvalContext::Bft)
}

pub fn deconstruct_account_transaction<P: chain_impl_mockchain::transaction::Payload>(
    transaction: &TransactionSlice<P>,
) -> Result<(P, account::Identifier, account::Witness), ValidationError> {
    let payload = transaction.payload().into_payload();
    let account = transaction.inputs().iter().next().unwrap().to_enum();

    let identifier = if let InputEnum::AccountInput(account, _) = account {
        account.to_single_account().unwrap()
    } else {
        return Err(ValidationError::InvalidUtxoInputs);
    };

    let witness =
        if let Witness::Account(_, witness) = transaction.witnesses().iter().next().unwrap() {
            witness
        } else {
            return Err(ValidationError::InvalidUtxoWitnesses);
        };

    Ok((payload, identifier, witness))
}

#[derive(thiserror::Error, Debug)]
pub enum ValidationError {
    #[error("Could not verify transaction {id} signature with range {range:?}")]
    InvalidTransactionSignature {
        id: String,
        range: std::ops::Range<u32>,
    },

    #[error("Invalid ballot, only 1 input (subsequently 1 witness) and no output is accepted")]
    InvalidVoteCast,

    #[error("Out of voting period")]
    VotingPeriodError,

    #[error("Out of tally period")]
    TallyPeriodError,

    #[error("Fragment should be either a votecast or a votetally")]
    NotAVotingFragment,

    #[error("Cannot handle utxo inputs")]
    InvalidUtxoInputs,

    #[error("Cannot handle utxo witnesses")]
    InvalidUtxoWitnesses,

    #[error("Fragment with id {id} and spending counter value was already processed")]
    DuplicatedFragment { id: FragmentId },

    #[error("Unsupported private votes")]
    UnsupportedPrivateVotes,

    #[error("Unbalanced transaction")]
    UnbalancedTransaction(#[from] chain_impl_mockchain::transaction::BalanceError),

    #[error("Transaction arrived before block0 start time")]
    TransactionBeforeStart,

    #[error("Transaction expiry date is too far in the future")]
    TransactionValidForTooLong,
}

#[derive(thiserror::Error, Debug)]
pub enum ReplayError {
    #[error("Account {0} is not known")]
    AccountNotFound(String),

    #[error("Multiple outputs for a single transaction are not supported")]
    UnsupportedMultipleOutputs,

    #[error("Tried to vote with a non registered account: {0}")]
    NonVotingAccount(String),

    #[error("Fragment with id {id} is not vote related")]
    NotAVotingFragment { id: String },
}

pub struct ValidatedFragment {
    pub fragment: Fragment,
    pub recorded_date: BlockDate,
    pub spending_counter: SpendingCounter,
}

pub struct ReplayedFragment {
    original: ValidatedFragment,
    replayed: Fragment,
}

pub struct VoteFragmentFilter<I: Iterator<Item = PersistentFragmentLog>> {
    block0: Hash,
    range_check: Range<u32>,
    timeframe: TimeFrame,
    fees: LinearFee,
    era: TimeEra,
    fragments: I,
    replay_protection: HashSet<FragmentId>,
    spending_counters: HashMap<account::Identifier, u32>,
}

impl<I: Iterator<Item = PersistentFragmentLog>> VoteFragmentFilter<I> {
    pub fn new(block0: Block, range_check: Range<u32>, fragments: I) -> Result<Self, Error> {
        let block0_configuration = Block0Configuration::from_block(&block0)?;
        let fees = block0_configuration.blockchain_configuration.linear_fees;
        let block0_start = block0_configuration.blockchain_configuration.block0_date;
        let slot_duration = block0_configuration.blockchain_configuration.slot_duration;
        let timeframe = timeframe_from_block0_start_and_slot_duration(block0_start, slot_duration);
        let era = TimeEra::new(
            Slot::from(0),
            Epoch(0),
            block0_configuration
                .blockchain_configuration
                .slots_per_epoch
                .into(),
        );
        Ok(Self {
            block0: block0.header().hash().into(),
            range_check,
            timeframe,
            era,
            fragments,
            fees,
            spending_counters: HashMap::new(),
            replay_protection: HashSet::new(),
        })
    }

    fn validate_tx<P: chain_impl_mockchain::transaction::Payload>(
        &mut self,
        transaction: &TransactionSlice<P>,
        fragment_id: FragmentId,
    ) -> Result<SpendingCounter, ValidationError> {
        // check if fragment was processed already
        if self.replay_protection.contains(&fragment_id) {
            return Err(ValidationError::DuplicatedFragment { id: fragment_id });
        }

        let (_, identifier, witness) = deconstruct_account_transaction(transaction)?;

        transaction.verify_strictly_balanced(self.fees.calculate_tx(transaction))?;

        let spending_counter = self
            .spending_counters
            .entry(identifier.clone())
            .or_default();

        let (valid, sc) = verify_original_tx(
            SpendingCounter::from(*spending_counter),
            &self.block0.into_hash(),
            &transaction.transaction_sign_data_hash(),
            &identifier,
            &witness,
            self.range_check.clone(),
        );

        if !valid {
            return Err(ValidationError::InvalidTransactionSignature {
                id: fragment_id.to_string(),
                range: self.range_check.clone(),
            });
        }

        self.replay_protection.insert(fragment_id);
        *self.spending_counters.get_mut(&identifier).unwrap() += 1;
        Ok(sc)
    }
}

impl<I: Iterator<Item = PersistentFragmentLog>> Iterator for VoteFragmentFilter<I> {
    type Item = Result<ValidatedFragment, (Fragment, ValidationError)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.fragments.next().map(|persistent_fragment_log| {
            let PersistentFragmentLog { fragment, time } = persistent_fragment_log;
            let spending_counter = match &fragment {
                Fragment::VoteCast(tx) => {
                    let transaction_slice = tx.as_slice();
                    let is_valid_vote_cast = valid_vote_cast(&transaction_slice);
                    if !is_valid_vote_cast {
                        return Err((fragment, ValidationError::InvalidVoteCast));
                    }

                    self.validate_tx(&transaction_slice, fragment.id())
                }
                Fragment::VoteTally(tx) => self.validate_tx(&tx.as_slice(), fragment.id()),
                Fragment::Transaction(tx) => self.validate_tx(&tx.as_slice(), fragment.id()),
                _ => Err(ValidationError::NotAVotingFragment),
            }
            .map_err(|e| (fragment.clone(), e))?;

            let recorded_date =
                fragment_log_timestamp_to_blockdate(time, &self.timeframe, &self.era)
                    .ok_or((fragment.clone(), ValidationError::TransactionBeforeStart))?;

            Ok(ValidatedFragment {
                fragment,
                recorded_date,
                spending_counter,
            })
        })
    }
}

pub fn recover_ledger_from_logs(
    block0: &Block,
    fragment_logs: impl Iterator<Item = Result<PersistentFragmentLog, FragmentLogDeserializeError>>,
) -> Result<(Ledger, Vec<Fragment>), Error> {
    let (mut fragment_replayer, new_block0) = FragmentReplayer::from_block0(block0)?;

    // we use block0 header id instead of the new one, to keep validation on old tx that uses the original block0 id.
    // This is used so we can run the VoteTally certificates with the original (issued) committee members ones.
    let mut ledger =
        Ledger::new(block0.header().id(), new_block0.fragments()).map_err(Error::LedgerError)?;

    // deserialize fragments to get a clean iterator over them
    let deserialized_fragment_logs = fragment_logs.filter_map(|fragment_log| match fragment_log {
        Ok(fragment) => Some(fragment),
        Err(e) => {
            error!("Error deserializing PersistentFragmentLog: {:?}", e);
            None
        }
    });

    // use double of proposals range as possible spending counters to check
    let spending_counter_max_check: u32 = voteplans_from_block0(block0)
        .values()
        .flat_map(|voteplan| voteplan.proposals().iter())
        .count() as u32
        * 2;

    let fragment_filter = VoteFragmentFilter::new(
        block0.clone(),
        0..spending_counter_max_check,
        deserialized_fragment_logs,
    )?;
    let mut failed_fragments = Vec::new();
    let mut current_date = BlockDate::first();
    for filtered_fragment in fragment_filter {
        let new_fragment = filtered_fragment
            .map_err(|(fragment, err)| (Error::from(err), fragment))
            .and_then(|fragment| fragment_replayer.replay(fragment))
            .and_then(|fragment| {
                // It may happen, though is unlikely, that fragment current slots are not monotonic, due to adjustments
                // in the underlying clock of the node. We assume that such corrections are small and ignore any
                // block date that is before our current one.
                let ReplayedFragment { original, replayed } = fragment;
                let date = original.recorded_date;
                if date > current_date {
                    ledger = increment_ledger_time_up_to(&ledger, date);
                    current_date = date;
                }

                ledger
                    .apply_fragment(&ledger.get_ledger_parameters(), &replayed, current_date)
                    .map(|ledger| (ledger, replayed))
                    .map_err(|e| (Error::from(e), original.fragment))
            });

        match new_fragment {
            Ok((new_ledger, fragment)) => {
                ledger = new_ledger;
                fragment_replayer.confirm_fragment(&fragment);
            }
            Err((
                err @ Error::LedgerError(ledger::Error::VotePlan(_) | ledger::Error::InvalidTransactionValidity(_) | ledger::Error::Account(LedgerError::ValueError(
                    ValueError::NegativeAmount,
                )))
                | err @ Error::ValidationError(_)
                | err @ Error::ReplayError(_),
                fragment,
            )) => {
                warn!("Invalid fragment detected: {:?}", err);
                failed_fragments.push(fragment);
            }
            Err(e) => unreachable!("Should be impossible to fail, since we should be using proper spending counters and signatures {:?}", e)
        }
    }

    Ok((ledger, failed_fragments))
}

struct FragmentReplayer {
    wallets: HashMap<Address, Wallet>,
    non_voting_wallets: HashMap<Address, Wallet>,
    pending_requests: HashMap<FragmentId, Address>,

    settings: Settings,
}

impl FragmentReplayer {
    // build a new block0 with mirror accounts and same configuration as original one
    fn from_block0(block0: &Block) -> Result<(Self, Block), Error> {
        let mut config =
            Block0Configuration::from_block(block0).map_err(Error::Block0ConfigurationError)?;

        let mut wallets = HashMap::new();
        let mut rng = rand::thread_rng();

        let committee_members = config
            .blockchain_configuration
            .committees
            .iter()
            .cloned()
            .map(committee_id_to_address)
            .collect::<HashSet<_>>();

        for initial in &mut config.initial {
            if let Initial::Fund(ref mut utxos) = initial {
                let mut new_committee_accounts = Vec::new();
                for utxo in utxos.iter_mut() {
                    let mut wallet =
                        Wallet::new_from_key(<SecretKey<Ed25519Extended>>::generate(&mut rng));
                    let new_initial_utxo = InitialUTxO {
                        address: wallet
                            .account_id()
                            .address(Discrimination::Production)
                            .into(),
                        value: utxo.value,
                    };
                    wallet.update_state(utxo.value.into(), 0.into());
                    wallets.insert(utxo.address.clone(), wallet);
                    if committee_members.contains(&utxo.address) {
                        trace!("Committee account found {}", &utxo.address);
                        // push new mirror address
                        new_committee_accounts.push(new_initial_utxo);
                    } else {
                        *utxo = new_initial_utxo;
                    }
                }
                utxos.append(&mut new_committee_accounts);
            }

            if let Initial::Token(ref mut mint) = initial {
                for destination in mint.to.iter_mut() {
                    destination.address = wallets[&destination.address]
                        .account_id()
                        .address(Discrimination::Production)
                        .into();
                }
            }
        }

        Ok((
            Self {
                wallets,
                non_voting_wallets: HashMap::new(),
                settings: Settings::new(block0).unwrap(),
                pending_requests: HashMap::new(),
            },
            config.to_block(),
        ))
    }

    fn replay_votecast(&mut self, tx: TransactionSlice<VoteCast>) -> Result<Fragment, Error> {
        let (vote_cast, identifier, _) = deconstruct_account_transaction(&tx)?;
        let address =
            Identifier::from(identifier).to_address(chain_addr::Discrimination::Production);

        let address: Address = address.into();
        let wallet = self
            .wallets
            .get_mut(&address)
            .ok_or_else(|| ReplayError::NonVotingAccount(address.to_string()))?;

        // unwrap checked in the validation step
        let builder_help = wallet.new_transaction(tx.total_input().unwrap()).unwrap();
        let mut builder = TransactionBuilder::new(&self.settings, vote_cast, tx.valid_until());
        builder.add_input(builder_help.input(), builder_help.witness_builder());
        let res = Fragment::VoteCast(builder.finalize_tx(()).unwrap());

        debug!("replaying vote cast transaction from {}", address);
        self.pending_requests.insert(res.id(), address);
        Ok(res)
    }

    fn replay_tx(&mut self, tx: TransactionSlice<NoExtra>) -> Result<Fragment, Error> {
        let (_, identifier, _) = deconstruct_account_transaction(&tx)?;
        let address =
            Identifier::from(identifier.clone()).to_address(chain_addr::Discrimination::Production);

        let address: Address = address.into();
        if tx.nb_outputs() != 1 {
            // The wallet lib we use does not corrently expose this functionality
            return Err(ReplayError::UnsupportedMultipleOutputs.into());
        }

        let output = tx.outputs().iter().next().unwrap();
        let output_address = if let Some(wlt) = self.wallets.get(&output.address.into()) {
            wlt.account_id()
        } else {
            self.non_voting_wallets
                .entry(address.clone())
                .or_insert_with(|| {
                    Wallet::new_from_key(<SecretKey<Ed25519Extended>>::generate(
                        &mut rand::thread_rng(),
                    ))
                })
                .account_id()
        }
        .address(Discrimination::Production);

        // Double self borrows are not allowed in closures, so this is written as an
        // if let instead of chaining methods on options
        let wallet = if let Some(wlt) = self.wallets.get_mut(&address) {
            wlt
        } else {
            self.non_voting_wallets
                .get_mut(&address)
                .ok_or_else(|| ReplayError::AccountNotFound(address.to_string()))?
        };

        warn!("replaying a plain transaction from {} to {:?} with value {}, this is not coming from the app, might want to look into this", identifier, output_address, output.value);
        // unwrap checked in the validation step
        let builder_help = wallet.new_transaction(tx.total_input().unwrap()).unwrap();
        let mut builder = TransactionBuilder::new(&self.settings, NoExtra, tx.valid_until());
        builder.add_input(builder_help.input(), builder_help.witness_builder());
        builder.add_output(Output::from_address(output_address, output.value));
        let res = Fragment::Transaction(builder.finalize_tx(()).unwrap());
        self.pending_requests.insert(res.id(), address);
        Ok(res)
    }

    // rebuild a fragment to be used in the new ledger configuration with the account mirror account.
    fn replay(
        &mut self,
        original: ValidatedFragment,
    ) -> Result<ReplayedFragment, (Error, Fragment)> {
        let replayed = match &original.fragment {
            Fragment::VoteCast(ref tx) => self.replay_votecast(tx.as_slice()),
            Fragment::Transaction(ref tx) => self.replay_tx(tx.as_slice()),
            fragment @ Fragment::VoteTally(_) => Ok(fragment.clone()),
            fragment => Err(ReplayError::NotAVotingFragment {
                id: fragment.id().to_string(),
            }
            .into()),
        }
        .map_err(|err| (err, original.fragment.clone()))?;
        Ok(ReplayedFragment { replayed, original })
    }

    fn confirm_fragment(&mut self, fragment: &Fragment) {
        if let Some(addr) = self.pending_requests.get(&fragment.id()) {
            if let Some(wallet) = self.wallets.get_mut(addr) {
                wallet.check_fragment(&fragment.id(), fragment);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::recover_ledger_from_logs;
    use assert_fs::fixture::PathChild;
    use assert_fs::TempDir;
    use chain_addr::Discrimination;
    use chain_core::property::Block as _;
    use chain_impl_mockchain::certificate::VoteTallyPayload;
    use chain_impl_mockchain::chaintypes::ConsensusType;
    use chain_impl_mockchain::tokens::minting_policy::MintingPolicy;
    use chain_impl_mockchain::vote::Choice;
    use chain_impl_mockchain::vote::Tally;
    use jormungandr_automation::jormungandr::ConfigurationBuilder;
    use jormungandr_automation::testing::VotePlanBuilder;
    use jormungandr_lib::interfaces::load_persistent_fragments_logs_from_folder_path;
    use jormungandr_lib::interfaces::InitialToken;
    use jormungandr_lib::interfaces::KesUpdateSpeed;
    use jormungandr_lib::interfaces::PersistentFragmentLog;
    use jormungandr_lib::time::SecondsSinceUnixEpoch;
    use rand::rngs::OsRng;
    use thor::vote_plan_cert;
    use thor::write_into_persistent_log;
    use thor::FragmentBuilder;
    use thor::Wallet as TestWallet;

    #[test]
    fn test_vote_flow() {
        let temp_dir = TempDir::new().unwrap();
        let funds = 1_000_000;
        let slot_duration = 4;
        let slots_per_epoch = 10;

        let mut rng = OsRng;
        let alice =
            TestWallet::new_account_with_discrimination(&mut rng, Discrimination::Production);
        let bob = TestWallet::new_account_with_discrimination(&mut rng, Discrimination::Production);
        let clarice =
            TestWallet::new_account_with_discrimination(&mut rng, Discrimination::Production);

        let vote_plan = VotePlanBuilder::new().proposals_count(3).public().build();

        let vote_plan_cert = vote_plan_cert(
            &alice,
            chain_impl_mockchain::block::BlockDate {
                epoch: 1,
                slot_id: 0,
            },
            &vote_plan,
        )
        .into();

        let minting_policy = MintingPolicy::new();
        let token_id = vote_plan.voting_token();

        let block0_configuration = ConfigurationBuilder::new()
            .with_explorer()
            .with_funds(vec![
                alice.to_initial_fund(funds),
                bob.to_initial_fund(funds),
                clarice.to_initial_fund(funds),
            ])
            .with_token(InitialToken {
                token_id: token_id.clone().into(),
                policy: minting_policy.into(),
                to: vec![
                    alice.to_initial_token(funds),
                    bob.to_initial_token(funds),
                    clarice.to_initial_token(funds),
                ],
            })
            .with_certs(vec![vote_plan_cert])
            .with_block0_consensus(ConsensusType::Bft)
            .with_kes_update_speed(KesUpdateSpeed::new(43200).unwrap())
            .with_discrimination(Discrimination::Production)
            .with_committees(&[alice.to_committee_id()])
            .with_slot_duration(slot_duration)
            .with_slots_per_epoch(slots_per_epoch)
            .build_block0();

        let valid_until = chain_impl_mockchain::block::BlockDate {
            epoch: 1,
            slot_id: 0,
        };
        let block0 = block0_configuration.to_block();

        let fragment_builder = FragmentBuilder::new(
            &block0.id().into(),
            &block0_configuration.blockchain_configuration.linear_fees,
            valid_until,
        );

        //vote fragments
        let mut fragments: Vec<PersistentFragmentLog> = vec![
            fragment_builder.vote_cast(&alice, &vote_plan, 0, &Choice::new(1)),
            fragment_builder.vote_cast(&bob, &vote_plan, 1, &Choice::new(1)),
            fragment_builder.vote_cast(&clarice, &vote_plan, 2, &Choice::new(1)),
        ]
        .into_iter()
        .map(|fragment| PersistentFragmentLog {
            time: block0_configuration.blockchain_configuration.block0_date,
            fragment,
        })
        .collect();

        let valid_until = chain_impl_mockchain::block::BlockDate {
            epoch: 2,
            slot_id: 0,
        };

        let fragment_builder = FragmentBuilder::new(
            &block0.id().into(),
            &block0_configuration.blockchain_configuration.linear_fees,
            valid_until,
        );

        //calculate tally period
        let new_secs = block0_configuration
            .blockchain_configuration
            .block0_date
            .to_secs()
            + (slots_per_epoch * slot_duration as u32 + 1) as u64;

        //push tally fragment
        fragments.push(PersistentFragmentLog {
            time: SecondsSinceUnixEpoch::from_secs(new_secs),
            fragment: fragment_builder.vote_tally(&alice, &vote_plan, VoteTallyPayload::Public),
        });

        let persistent_fragment_log_output = temp_dir.child("fragments");
        std::fs::create_dir_all(persistent_fragment_log_output.path()).unwrap();

        let fragment_log = persistent_fragment_log_output.child("log.log");
        // below loop of writing/reading is done not to loose original test
        write_into_persistent_log(fragment_log.path(), fragments).unwrap();
        let fragments =
            load_persistent_fragments_logs_from_folder_path(persistent_fragment_log_output.path())
                .unwrap();

        let block0 = block0_configuration.to_block();
        let (ledger, failed) = recover_ledger_from_logs(&block0, fragments).unwrap();

        assert_eq!(failed.len(), 0, "Failed: {}", failed.len());
        for voteplan in ledger.active_vote_plans() {
            println!("Voteplan: {}", voteplan.id);
            for proposal in voteplan.proposals {
                match proposal.tally {
                    Tally::Public { result } => {
                        let results = result.results();
                        assert_eq!(*results.get(0).unwrap(), 0.into());
                        assert_eq!(*results.get(1).unwrap(), funds.into());
                        assert_eq!(*results.get(2).unwrap(), 0.into());
                    }
                    Tally::Private { .. } => {
                        unimplemented!("Private tally testing is not implemented")
                    }
                }
            }
        }
    }
}
