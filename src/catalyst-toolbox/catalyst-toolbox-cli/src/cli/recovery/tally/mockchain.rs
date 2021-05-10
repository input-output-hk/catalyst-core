use std::ops::{Add, Range};
use std::time::{Duration, SystemTime};

use super::Error;
use chain_addr::{Discrimination, Kind};
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

fn timestamp_to_system_time(ts: SecondsSinceUnixEpoch) -> SystemTime {
    SystemTime::UNIX_EPOCH.add(Duration::new(ts.to_secs(), 0))
}

fn fragment_log_timestamp_to_blockdate(
    timestamp: SecondsSinceUnixEpoch,
    timeframe: &TimeFrame,
    ledger: &Ledger,
) -> BlockDate {
    let slot = timestamp_to_system_time(timestamp);
    // TODO: Get rid of unwraps
    let new_slot = timeframe.slot_at(&slot).unwrap();
    let epoch_position = ledger.era().from_slot_to_era(new_slot).unwrap();
    BlockDate::from(epoch_position)
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

fn verify_original_tx(
    spending_counter: SpendingCounter,
    block0_hash: &HeaderId,
    sign_data_hash: &TransactionSignDataHash,
    account: &account::Identifier,
    witness: &account::Witness,
    range_check: Range<i32>,
) -> (bool, u32) {
    for i in range_check {
        let spending_counter: i32 = <u32>::from(spending_counter) as i32;
        let new_spending_counter = spending_counter.add(i).clamp(0, i32::MAX) as u32;
        let tidsc = WitnessAccountData::new(
            block0_hash,
            sign_data_hash,
            SpendingCounter::from(new_spending_counter),
        );
        if witness.verify(account.as_ref(), &tidsc) == chain_crypto::Verification::Success {
            println!(
                "expected: {} found: {}",
                spending_counter, new_spending_counter
            );
            return (true, new_spending_counter);
        }
    }
    (false, 0)
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
    println!("{}", vote_start);

    let timeframe = timeframe_from_block0_start_and_slot_duration(block0_start, slot_duration);

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
    let mut inc_tally = true;
    for fragment_log in fragment_logs {
        match fragment_log {
            Ok(PersistentFragmentLog { fragment, time }) => {
                let block_date = fragment_log_timestamp_to_blockdate(time, &timeframe, &ledger);

                println!("Fragment processed {}", fragment.hash());
                let new_fragment = match &fragment {
                    Fragment::VoteCast(_) => {
                        if let Ok(new_fragment) = fragment_replayer
                            .replay(fragment.clone())
                            .map_err(|e| println!("Fragment couldn't be processed:\n\t {:?}", e))
                        {
                            if vote_start > block_date || vote_end <= block_date {
                                unimplemented!(
                                        "Explain that fragment is skipped because it is out of vote time"
                                    );
                            }
                            Some(new_fragment)
                        } else {
                            failed_fragments.push(fragment);
                            None
                        }
                    }
                    new_fragment @ Fragment::VoteTally(_) => {
                        if inc_tally {
                            ledger = ledger
                                .begin_block(
                                    ledger.get_ledger_parameters(),
                                    ledger.chain_length().increase(),
                                    vote_end,
                                )
                                .unwrap()
                                .finish(&ConsensusEvalContext::Bft);
                            inc_tally = false;
                        }
                        Some(new_fragment.clone())
                    }
                    _ => None,
                };
                if let Some(new_fragment) = new_fragment {
                    ledger = ledger.apply_fragment(&ledger.get_ledger_parameters(), &new_fragment, block_date)
                        .expect("Should be impossible to fail, since we would be using proper spending counters and signatures");
                }
            }
            Err(_e) => {
                unimplemented!("Dump error")
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
    const CHECK_RANGE: Range<i32> = -50..50;

    fn from_block0(block0: &Block) -> Result<(Self, Block), Error> {
        let mut config =
            Block0Configuration::from_block(block0).map_err(Error::Block0ConfigurationError)?;
        let voteplans = block0
            .fragments()
            .filter_map(|fragment| {
                if let Fragment::VotePlan(tx) = fragment {
                    let voteplan = tx.as_slice().payload().into_payload();
                    Some((voteplan.to_id(), voteplan))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>();

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
                        println!("Committee account found {}", &utxo.address);
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
                panic!("this cannot happen")
            };
            if transaction_slice.nb_witnesses() != 1 {
                unimplemented!("Multisig not implemented");
            }
            let witness = if let Witness::Account(witness) =
                transaction_slice.witnesses().iter().next().unwrap()
            {
                witness
            } else {
                panic!("utxo witnesses not supported");
            };

            let (valid, sc) = verify_original_tx(
                spending_counter,
                &self.old_block0_hash.into_hash(),
                &sign_data_hash,
                &identifier.to_inner(),
                &witness,
                Self::CHECK_RANGE,
            );

            if !valid {
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
    use crate::cli::recovery::tally::mockchain::recover_ledger_from_logs;
    use chain_impl_mockchain::block::Block;
    use chain_ser::deser::Deserialize;
    use jormungandr_lib::interfaces::load_persistent_fragments_logs_from_folder_path;
    use std::io::BufReader;
    use std::path::PathBuf;

    fn read_block0(path: PathBuf) -> std::io::Result<Block> {
        let reader = std::fs::File::open(path)?;
        Ok(Block::deserialize(BufReader::new(reader)).unwrap())
    }

    #[test]
    fn test_vote_flow() -> std::io::Result<()> {
        let path: PathBuf = r"/Users/daniel/projects/rust/catalyst-toolbox/testing/logs"
            .parse()
            .unwrap();

        let fragments = load_persistent_fragments_logs_from_folder_path(&path)?;

        let block0 =
            read_block0(r"/Users/daniel/projects/rust/catalyst-toolbox/testing/block0.bin".into())?;

        let (ledger, failed) = recover_ledger_from_logs(&block0, fragments).unwrap();

        println!("Failed: {}", failed.len());
        for voteplan in ledger.active_vote_plans() {
            for proposal in voteplan.proposals {
                println!("{:?}", proposal.tally);
            }
        }

        Ok(())
    }
}
