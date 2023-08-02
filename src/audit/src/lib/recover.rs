#![allow(clippy::result_large_err)]
use chain_addr::{Discrimination, Kind};
use chain_core::property::Fragment as _;
use chain_crypto::{Ed25519Extended, SecretKey};
use chain_impl_mockchain::{
    account::{self, LedgerError, SpendingCounter},
    accounting::account::SpendingCounterIncreasing,
    block::{Block, BlockDate},
    certificate::{self, VoteCast},
    fee::{FeeAlgorithm, LinearFee},
    fragment::{Fragment, FragmentId},
    ledger::{self, Ledger},
    transaction::{InputEnum, NoExtra, Output, TransactionSlice, Witness},
    value::ValueError,
    vote::CommitteeId,
};

use jormungandr_lib::{
    crypto::account::Identifier,
    interfaces::{Address, Block0Configuration, CommitteeIdDef, Initial, InitialUTxO},
};
use std::collections::{HashMap, HashSet};

use tracing::{debug, error, trace, warn};
use wallet::{transaction::WitnessInput, Settings, TransactionBuilder, Wallet};

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

    #[error(transparent)]
    WalletError(#[from] wallet::Error),

    #[error(transparent)]
    TxBuilderError(#[from] wallet::transaction::Error),

    #[error(transparent)]
    ValueError(#[from] ValueError),
}

fn committee_id_to_address(id: CommitteeIdDef) -> Address {
    let id = CommitteeId::from(id);
    let pk = id.public_key();
    chain_addr::Address(Discrimination::Production, Kind::Account(pk)).into()
}

/// check that the transaction input/outputs/witnesses is valid for the ballot
/// * Only 1 input (subsequently 1 witness), no output
pub(crate) fn valid_vote_cast(tx: &TransactionSlice<certificate::VoteCast>) -> bool {
    !(tx.inputs().nb_inputs() != 1
        || tx.witnesses().nb_witnesses() != 1
        || tx.outputs().nb_outputs() != 0)
}

/// Unpack TX into payload, id and witness
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

/// Fragment which has passed validation
pub struct ValidatedFragment {
    pub fragment: Fragment,
}

pub struct ReplayedFragment {
    original: ValidatedFragment,
    replayed: Fragment,
}

pub struct VoteFragmentFilter<I: Iterator<Item = Fragment>> {
    fees: LinearFee,
    fragments: I,
    replay_protection: HashSet<FragmentId>,
}

impl<I: Iterator<Item = Fragment>> VoteFragmentFilter<I> {
    pub fn new(block0: Block, fragments: I) -> Result<Self, Error> {
        let block0_configuration = Block0Configuration::from_block(&block0)?;
        let fees = block0_configuration.blockchain_configuration.linear_fees;

        Ok(Self {
            fragments,
            fees,
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

        let (_, _identifier, _witness) = deconstruct_account_transaction(transaction)?;

        transaction.verify_strictly_balanced(self.fees.calculate_tx(transaction))?;

        self.replay_protection.insert(fragment_id);

        Ok(SpendingCounter::zero())
    }
}

impl<I: Iterator<Item = Fragment>> Iterator for VoteFragmentFilter<I> {
    type Item = Result<ValidatedFragment, (Fragment, ValidationError)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.fragments.next().map(|fragment| {
            match &fragment {
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

            Ok(ValidatedFragment { fragment })
        })
    }
}

/// Replay fragments from storage and recover the ledger state
pub fn recover_ledger_from_fragments(
    block0: &Block,
    fragment_logs: impl Iterator<Item = Fragment>,
) -> Result<(Ledger, Vec<Fragment>), Error> {
    let (mut fragment_replayer, new_block0) = FragmentReplayer::from_block0(block0)?;

    // we use block0 header id instead of the new one, to keep validation on old tx that uses the original block0 id.
    // This is used so we can run the VoteTally certificates with the original (issued) committee members ones.
    let mut ledger =
        Ledger::new(block0.header().id(), new_block0.fragments()).map_err(Error::LedgerError)?;

    let fragment_filter = VoteFragmentFilter::new(block0.clone(), fragment_logs)?;
    let mut failed_fragments = Vec::new();
    let current_date = BlockDate::first();
    for filtered_fragment in fragment_filter {
        let new_fragment = filtered_fragment
            .map_err(|(fragment, err)| (Error::from(err), fragment))
            .and_then(|fragment| fragment_replayer.replay(fragment))
            .and_then(|fragment| {
                let ReplayedFragment { original, replayed } = fragment;

                ledger
                    .apply_fragment(&replayed, current_date)
                    .map(|ledger| (ledger, replayed))
                    .map_err(|e| (Error::from(e), original.fragment))
            });

        match new_fragment {
            Ok((new_ledger, fragment)) => {
                ledger = new_ledger;
                fragment_replayer.confirm_fragment(&fragment);
            }
            Err((
                err @ Error::LedgerError(
                    ledger::Error::VotePlan(_)
                    | ledger::Error::Account(LedgerError::ValueError(ValueError::NegativeAmount)),
                )
                | err @ Error::ValidationError(_)
                | err @ Error::ReplayError(_),
                fragment,
            )) => {
                warn!("Invalid fragment detected: {:?}", err);
                failed_fragments.push(fragment);
            }
            Err(e) => {
                warn!("Invalid fragment detected: {:?}", e);
                continue;
            }
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
                    wallet
                        .set_state(
                            utxo.value.into(),
                            SpendingCounterIncreasing::default().get_valid_counters(),
                        )
                        .expect("cannot update wallet state");
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

    // replay a fragment, and return the original fragment and the replayed one
    fn replay_votecast(&mut self, tx: TransactionSlice<VoteCast>) -> Result<Fragment, Error> {
        let (vote_cast, identifier, _) = deconstruct_account_transaction(&tx)?;
        let address =
            Identifier::from(identifier).to_address(chain_addr::Discrimination::Production);

        let address: Address = address.into();
        let wallet = self
            .wallets
            .get_mut(&address)
            .ok_or_else(|| ReplayError::NonVotingAccount(address.to_string()))?;

        let secret_key = wallet.secret_key();
        let builder_help = wallet.new_transaction(tx.total_input()?, 0)?;
        let mut builder =
            TransactionBuilder::new(self.settings.clone(), vote_cast, tx.valid_until());
        builder.add_input(builder_help.input(), builder_help.witness_builder());
        let res =
            Fragment::VoteCast(builder.finalize_tx((), vec![WitnessInput::SecretKey(secret_key)])?);

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
                    Wallet::new_from_key(<SecretKey<Ed25519Extended>>::generate(rand::thread_rng()))
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
        let secret_key = wallet.secret_key();
        let builder_help = wallet.new_transaction(tx.total_input()?, 0)?;
        let mut builder = TransactionBuilder::new(self.settings.clone(), NoExtra, tx.valid_until());
        builder.add_input(builder_help.input(), builder_help.witness_builder());
        builder.add_output(Output::from_address(output_address, output.value));
        let res = Fragment::Transaction(
            builder.finalize_tx((), vec![WitnessInput::SecretKey(secret_key)])?,
        );
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
                wallet.check_fragment(&fragment.id(), fragment).unwrap();
            }
        }
    }
}
