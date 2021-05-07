use std::ops::Add;
use std::time::{Duration, SystemTime};

use super::Error;
use chain_core::property::BlockDate as _;
use chain_impl_mockchain::certificate::{VotePlan, VotePlanId};
use chain_impl_mockchain::fee::LinearFee;
use chain_impl_mockchain::{
    block::{Block, BlockDate},
    fragment::Fragment,
    ledger::Ledger,
    transaction::InputEnum,
    vote::Payload,
};
use chain_time::{SlotDuration, TimeFrame, Timeline};
use jormungandr_lib::crypto::account::Identifier;
use jormungandr_lib::crypto::hash::Hash;
use jormungandr_lib::{
    interfaces::{
        Address, Block0Configuration, FragmentLogDeserializeError, Initial, PersistentFragmentLog,
        SlotDuration as Block0SlotDuration,
    },
    time::SecondsSinceUnixEpoch,
};
use jormungandr_testing_utils::wallet::Wallet;
use std::collections::HashMap;

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

pub fn recover_ledger_from_logs(
    block0: &Block,
    fragment_logs: impl Iterator<Item = Result<PersistentFragmentLog, FragmentLogDeserializeError>>,
) -> Result<Ledger, Error> {
    let block0_configuration = Block0Configuration::from_block(&block0).unwrap();

    let (mut fragment_replayer, new_block0) = FragmentReplayer::from_block0(&block0)?;

    let mut ledger =
        Ledger::new(new_block0.header.id(), new_block0.fragments()).map_err(Error::LedgerError)?;

    let block0_start = block0_configuration.blockchain_configuration.block0_date;
    let slot_duration = block0_configuration.blockchain_configuration.slot_duration;
    let fees = block0_configuration.blockchain_configuration.linear_fees;

    // we assume that voteplans use the same vote start/end BlockDates as well as committee and tally ones
    // hence we only take data from one of them
    let voteplan = ledger
        .active_vote_plans()
        .last()
        .ok_or(Error::MissingVoteplanError)?;
    let vote_start = voteplan.vote_start;
    let vote_end = voteplan.vote_end;

    let timeframe = timeframe_from_block0_start_and_slot_duration(block0_start, slot_duration);

    for fragment_log in fragment_logs {
        match fragment_log {
            Ok(PersistentFragmentLog { fragment, time }) => {
                let block_date = fragment_log_timestamp_to_blockdate(time, &timeframe, &ledger);
                let new_fragment = fragment_replayer.replay(fragment.clone());
                if matches!(fragment, Fragment::VoteCast(_)) {
                    if vote_start > block_date || vote_end <= block_date {
                        unimplemented!(
                            "Exaplain that fragment is skipped because it is out of vote time"
                        )
                    }
                }
            }
            Err(e) => {
                unimplemented!("Dump error")
            }
        }
    }

    Ok(ledger)
}

struct FragmentReplayer {
    wallets: HashMap<Address, Wallet>,
    voteplans: HashMap<VotePlanId, VotePlan>,
    block0_hash: Hash,
    fees: LinearFee,
}

impl FragmentReplayer {
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
        for initial in &mut config.initial {
            if let Initial::Fund(mut utxos) = initial {
                for utxo in &mut utxos {
                    let wallet = Wallet::new_account_with_discrimination(
                        &mut rng,
                        chain_addr::Discrimination::Production,
                    );
                    let new_initial_utxo = wallet.to_initial_fund(utxo.value.into());
                    wallets.insert(utxo.address, wallet);
                    *utxo = new_initial_utxo;
                }
            }
        }
        let block0_hash: Hash = block0.header.id().into();
        let fees = config.blockchain_configuration.linear_fees;

        Ok((
            Self {
                wallets,
                voteplans,
                block0_hash,
                fees,
            },
            config.to_block(),
        ))
    }

    fn replay(&mut self, fragment: Fragment) -> Result<Fragment, Error> {
        if let Fragment::VoteCast(transaction) = fragment {
            let vote_cast = transaction.as_slice().payload().into_payload();
            let account = transaction
                .as_slice()
                .inputs()
                .iter()
                .next()
                .unwrap()
                .to_enum();

            let address = if let InputEnum::AccountInput(account, _) = account {
                Identifier::from(account.to_single_account().unwrap())
                    .to_address(chain_addr::Discrimination::Production)
            } else {
                panic!("cannot handle utxo inputs");
            };

            let choice = if let Payload::Public { choice } = vote_cast.payload() {
                choice
            } else {
                panic!("cannot handle private votes");
            };

            let wallet = self.wallets.get_mut(&address).unwrap();
            let vote_plan = self
                .voteplans
                .get(vote_cast.vote_plan())
                .ok_or(Error::MissingVoteplanError)?;
            let res = wallet
                .issue_vote_cast_cert(
                    &self.block0_hash,
                    &self.fees,
                    &vote_plan,
                    vote_cast.proposal_index(),
                    &choice,
                )
                .unwrap();
            wallet.confirm_transaction();
            Ok(res)
        } else {
            unimplemented!()
        }
    }
}
