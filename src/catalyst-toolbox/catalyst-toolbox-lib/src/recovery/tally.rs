use std::ops::{Add, Range};
use std::time::{Duration, SystemTime};

use chain_addr::{Discrimination, Kind};
use chain_core::property::Fragment as _;
use chain_impl_mockchain::account::SpendingCounter;
use chain_impl_mockchain::block::HeaderId;
use chain_impl_mockchain::certificate::{VotePlan, VotePlanId};
use chain_impl_mockchain::chaineval::ConsensusEvalContext;
use chain_impl_mockchain::fee::LinearFee;
use chain_impl_mockchain::transaction::{TransactionSignDataHash, Witness, WitnessAccountData};
use chain_impl_mockchain::{
    account,
    block::{Block, BlockDate},
    fragment::Fragment,
    ledger::Ledger,
    transaction::InputEnum,
    vote::{CommitteeId, Payload},
};
use chain_time::{SlotDuration, TimeFrame, Timeline};
use jormungandr_lib::crypto::account::Identifier;
use jormungandr_lib::crypto::hash::Hash;
use jormungandr_lib::interfaces::CommitteeIdDef;
use jormungandr_lib::{
    interfaces::{
        Address, Block0Configuration, FragmentLogDeserializeError, Initial, PersistentFragmentLog,
        SlotDuration as Block0SlotDuration,
    },
    time::SecondsSinceUnixEpoch,
};
use jormungandr_testing_utils::wallet::Wallet;
use std::collections::{HashMap, HashSet};

#[allow(clippy::large_enum_variant)]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DeserializeError(#[from] jormungandr_lib::interfaces::FragmentLogDeserializeError),

    #[error(transparent)]
    LedgerError(#[from] chain_impl_mockchain::ledger::Error),

    #[error("Couldn't initiate a new wallet")]
    WalletError(#[from] jormungandr_testing_utils::wallet::WalletError),

    #[error(transparent)]
    Block0ConfigurationError(#[from] jormungandr_lib::interfaces::Block0ConfigurationError),

    #[error("Block0 do not contain any voteplan")]
    MissingVoteplanError,

    #[error("Could not verify transaction {id} signature with range {range:?}")]
    InvalidTransactionSignature {
        id: String,
        range: std::ops::Range<u32>,
    },
}

fn timestamp_to_system_time(ts: SecondsSinceUnixEpoch) -> SystemTime {
    SystemTime::UNIX_EPOCH.add(Duration::new(ts.to_secs(), 0))
}

fn fragment_log_timestamp_to_blockdate(
    timestamp: SecondsSinceUnixEpoch,
    timeframe: &TimeFrame,
    ledger: &Ledger,
) -> Option<BlockDate> {
    let slot = timestamp_to_system_time(timestamp);
    let new_slot = timeframe.slot_at(&slot)?;
    let epoch_position = ledger.era().from_slot_to_era(new_slot)?;
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

fn verify_original_tx(
    spending_counter: SpendingCounter,
    block0_hash: &HeaderId,
    sign_data_hash: &TransactionSignDataHash,
    account: &account::Identifier,
    witness: &account::Witness,
    range_check: Range<u32>,
) -> (bool, u32) {
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
                    eprintln!(
                        "expected: {} found: {}",
                        spending_counter, new_spending_counter
                    );
                    return (true, new_spending_counter);
                }
            }
        }
    }
    (false, 0)
}

fn increment_ledger_time_up_to(ledger: Ledger, blockdate: BlockDate) -> Ledger {
    ledger
        .begin_block(
            ledger.get_ledger_parameters(),
            ledger.chain_length().increase(),
            blockdate,
        )
        .unwrap()
        .finish(&ConsensusEvalContext::Bft)
}

pub fn recover_ledger_from_logs(
    block0: &Block,
    fragment_logs: impl Iterator<Item = Result<PersistentFragmentLog, FragmentLogDeserializeError>>,
) -> Result<(Ledger, Vec<Fragment>), Error> {
    let block0_configuration = Block0Configuration::from_block(block0).unwrap();
    let mut failed_fragments = Vec::new();

    let (mut fragment_replayer, new_block0) = FragmentReplayer::from_block0(block0)?;

    // we use block0 header id instead of the new one, to keep validation on old tx that uses the original block0 id.
    // This is used so we can run the VoteTally certificates with the original (issued) committee members ones.
    let mut ledger =
        Ledger::new(block0.header.id(), new_block0.fragments()).map_err(Error::LedgerError)?;

    let block0_start = block0_configuration.blockchain_configuration.block0_date;
    let slot_duration = block0_configuration.blockchain_configuration.slot_duration;

    // we assume that voteplans use the same vote start/end BlockDates as well as committee and tally ones
    // hence we only take data from one of them
    let active_voteplans = ledger.active_vote_plans();
    let voteplan = active_voteplans.last().ok_or(Error::MissingVoteplanError)?;
    let vote_start = voteplan.vote_start;
    let vote_end = voteplan.vote_end;

    let timeframe = timeframe_from_block0_start_and_slot_duration(block0_start, slot_duration);

    // do not update ledger epoch if we are already in expected Epoch(0)
    if vote_start.epoch != 0 {
        ledger = ledger
            .begin_block(
                ledger.get_ledger_parameters(),
                ledger.chain_length().increase(),
                vote_start,
            )
            .unwrap()
            .finish(&ConsensusEvalContext::Bft);
    }

    // flag to check if we already incremented the time for tally (advance to vote_end)
    let mut inc_tally = true;
    for fragment_log in fragment_logs {
        match fragment_log {
            Ok(PersistentFragmentLog { fragment, time }) => {
                let block_date = fragment_log_timestamp_to_blockdate(time, &timeframe, &ledger)
                    .expect("BlockDates should always be valid for logs timestamps");

                eprintln!("Fragment processed {}", fragment.hash());
                let new_fragment = match &fragment {
                    fragment @ Fragment::VoteCast(_) => {
                        if let Ok(new_fragment) =
                            fragment_replayer.replay(fragment.clone()).map_err(|e| {
                                eprintln!(
                                    "Fragment {} couldn't be processed:\n\t {:?}",
                                    fragment.id(),
                                    e
                                );
                            })
                        {
                            if vote_start > block_date || vote_end <= block_date {
                                eprintln!(
                                    "Fragment {} skipped because it was out of voting time ({}-{}-{})",
                                    fragment.id(),
                                    vote_start,
                                    block_date,
                                    vote_end,
                                );
                                None
                            } else {
                                Some(new_fragment)
                            }
                        } else {
                            failed_fragments.push(fragment.clone());
                            None
                        }
                    }
                    new_fragment @ Fragment::VoteTally(_) => {
                        if inc_tally {
                            ledger = increment_ledger_time_up_to(ledger, vote_end);
                            inc_tally = false;
                        }
                        Some(new_fragment.clone())
                    }
                    _ => None,
                };
                if let Some(new_fragment) = new_fragment {
                    ledger = ledger.apply_fragment(&ledger.get_ledger_parameters(), &new_fragment, block_date)
                        .expect("Should be impossible to fail, since we should be using proper spending counters and signatures");
                }
            }
            Err(e) => {
                eprintln!("Error deserializing PersistentFragmentLog: {:?}", e);
            }
        }
    }
    Ok((ledger, failed_fragments))
}

struct FragmentReplayer {
    wallets: HashMap<Address, Wallet>,
    spending_counters: HashMap<Address, Vec<u32>>,
    voteplans: HashMap<VotePlanId, VotePlan>,
    old_block0_hash: Hash,
    fees: LinearFee,
}

impl FragmentReplayer {
    const CHECK_RANGE: Range<u32> = 0..50;

    // build a new block0 with mirror accounts and same configuration as original one
    fn from_block0(block0: &Block) -> Result<(Self, Block), Error> {
        let mut config =
            Block0Configuration::from_block(block0).map_err(Error::Block0ConfigurationError)?;

        let voteplans = voteplans_from_block0(&block0);

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
                for utxo in utxos.iter_mut() {
                    let wallet = Wallet::new_account_with_discrimination(
                        &mut rng,
                        chain_addr::Discrimination::Production,
                    );
                    if committee_members.contains(&utxo.address) {
                        eprintln!("Committee account found {}", &utxo.address);
                        continue;
                    }
                    let new_initial_utxo = wallet.to_initial_fund(utxo.value.into());
                    wallets.insert(utxo.address.clone(), wallet);
                    *utxo = new_initial_utxo;
                }
            }
        }

        let fees = config.blockchain_configuration.linear_fees;

        Ok((
            Self {
                wallets,
                spending_counters: HashMap::new(),
                voteplans,
                old_block0_hash: block0.header.id().into(),
                fees,
            },
            config.to_block(),
        ))
    }

    // rebuild a fragment to be used in the new ledger configuration with the account mirror account.
    fn replay(&mut self, fragment: Fragment) -> Result<Fragment, Error> {
        if let Fragment::VoteCast(ref transaction) = fragment {
            let transaction_slice = transaction.as_slice();

            let vote_cast = transaction_slice.payload().into_payload();
            let account = transaction
                .as_slice()
                .inputs()
                .iter()
                .next()
                .unwrap()
                .to_enum();

            let identifier = if let InputEnum::AccountInput(account, _) = account {
                Identifier::from(account.to_single_account().unwrap())
            } else {
                panic!("cannot handle utxo inputs");
            };
            let address = identifier.to_address(chain_addr::Discrimination::Production);

            let choice = if let Payload::Public { choice } = vote_cast.payload() {
                choice
            } else {
                panic!("cannot handle private votes");
            };

            let sign_data_hash = transaction_slice.transaction_sign_data_hash();
            let spending_counter = if let Wallet::Account(account) =
                self.wallets.get(&address.clone().into()).unwrap()
            {
                account.internal_counter()
            } else {
                panic!("New accounts spending counters should always be valid.")
            };

            if transaction_slice.nb_witnesses() != 1 {
                unimplemented!("Multi-signature is not supported");
            }

            let witness = if let Witness::Account(witness) =
                transaction_slice.witnesses().iter().next().unwrap()
            {
                witness
            } else {
                panic!("utxo witnesses not supported");
            };

            let (is_valid_tx, sc) = verify_original_tx(
                spending_counter,
                &self.old_block0_hash.into_hash(),
                &sign_data_hash,
                &identifier.to_inner(),
                &witness,
                Self::CHECK_RANGE,
            );

            if !is_valid_tx {
                return Err(Error::InvalidTransactionSignature {
                    id: fragment.clone().hash().to_string(),
                    range: Self::CHECK_RANGE,
                });
            }

            self.spending_counters
                .entry(address.clone().into())
                .or_insert_with(Vec::new)
                .push(sc);

            let wallet = self.wallets.get_mut(&address.into()).unwrap();

            let vote_plan = self
                .voteplans
                .get(vote_cast.vote_plan())
                .ok_or(Error::MissingVoteplanError)?;

            // we still use the old block0 hash because the new ledger will still use the old one for
            // verifications. This makes possible the usage of old VoteTally transactions without the need
            // to be replayed.
            let res = wallet
                .issue_vote_cast_cert(
                    &self.old_block0_hash,
                    &self.fees,
                    &vote_plan,
                    vote_cast.proposal_index(),
                    &choice,
                )
                .unwrap();
            wallet.confirm_transaction();
            Ok(res)
        } else {
            unimplemented!();
        }
    }
}

#[cfg(test)]
mod test {
    use super::{increment_ledger_time_up_to, recover_ledger_from_logs, voteplans_from_block0};
    use chain_impl_mockchain::block::Block;
    use chain_impl_mockchain::certificate::VoteTallyPayload;
    use chain_impl_mockchain::vote::Weight;
    use chain_ser::deser::Deserialize;
    use jormungandr_lib::interfaces::{
        load_persistent_fragments_logs_from_folder_path, Block0Configuration,
    };
    use jormungandr_testing_utils::wallet::Wallet;
    use std::io::BufReader;
    use std::path::PathBuf;

    fn read_block0(path: PathBuf) -> std::io::Result<Block> {
        let reader = std::fs::File::open(path)?;
        Ok(Block::deserialize(BufReader::new(reader)).unwrap())
    }

    #[test]
    fn test_vote_flow() -> std::io::Result<()> {
        println!("{}", std::env::current_dir().unwrap().to_string_lossy());
        let path = std::fs::canonicalize(r"../testing/logs").unwrap();
        println!(
            "{}",
            std::fs::canonicalize(path.clone())
                .unwrap()
                .to_string_lossy()
        );
        let fragments = load_persistent_fragments_logs_from_folder_path(&path)?;
        let block0_path: PathBuf = std::fs::canonicalize(r"../testing/block0.bin").unwrap();
        let block0 = read_block0(block0_path)?;
        let block0_configuration = Block0Configuration::from_block(&block0).unwrap();
        let (ledger, failed) = recover_ledger_from_logs(&block0, fragments).unwrap();
        let mut committee = Wallet::from_existing_account("ed25519e_sk1dpqkhtzyeaqvclvjf3hgdkw2rh5q06a2dqrp9qks32g96ta6k9alvhm7a0zp5j4gly90dmjj2w4ky3u86mpwxyctrc2k7s5qfq9dd8sefgey5", 0.into());
        let voteplans = voteplans_from_block0(&block0);
        let mut ledger =
            increment_ledger_time_up_to(ledger, voteplans.values().last().unwrap().vote_end());
        for (_, voteplan) in voteplans {
            let tally_cert = committee
                .issue_vote_tally_cert(
                    &block0.header.id().into(),
                    &block0_configuration.blockchain_configuration.linear_fees,
                    &voteplan,
                    VoteTallyPayload::Public,
                )
                .unwrap();
            ledger = ledger.apply_fragment(&ledger.get_ledger_parameters(), &tally_cert, ledger.date())
                .expect("Should be impossible to fail, since we should be using proper spending counters and signatures");
            committee.confirm_transaction();
        }

        println!("Failed: {}", failed.len());
        assert_eq!(failed.len(), 0);
        for voteplan in ledger.active_vote_plans() {
            println!("Voteplan: {}", voteplan.id);
            for proposal in voteplan.proposals {
                let result = proposal.tally.unwrap().result().cloned().unwrap();
                if result.results().iter().any(|w| w != &Weight::from(0)) {
                    println!("\tProposal: {}", proposal.proposal_id);
                    println!("\t\t{:?}", result.results());
                }
            }
        }

        Ok(())
    }
}
