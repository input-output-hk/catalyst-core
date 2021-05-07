use chain_core::property::BlockDate as _;
use chain_impl_mockchain::block::{Block, BlockDate, HeaderId};
use chain_impl_mockchain::fragment::Fragment;
use chain_impl_mockchain::ledger::Ledger;
use chain_time::{SlotDuration, TimeFrame, Timeline};
use jormungandr_lib::interfaces::{
    Block0Configuration, FragmentLogDeserializeError, PersistentFragmentLog,
};

use crate::cli::recovery::tally::Error;
use chain_impl_mockchain::chaineval::ConsensusEvalContext;
use jormungandr_lib::time::SecondsSinceUnixEpoch;
use std::collections::VecDeque;
use std::ops::Add;
use std::time::{Duration, SystemTime};

fn vote_start_date_from_ledger(ledger: &Ledger) -> (BlockDate, BlockDate) {
    ledger
        .active_vote_plans()
        .last()
        .map(|voteplan| (voteplan.vote_start, voteplan.vote_end))
        .unwrap()
}

fn timestamp_to_system_time(ts: SecondsSinceUnixEpoch) -> SystemTime {
    SystemTime::UNIX_EPOCH.add(Duration::new(ts.to_secs(), 0))
}

pub fn recover_ledger_from_fragments<'block0>(
    block0: &Block,
    block0_fragments: impl Iterator<Item = &'block0 Fragment>,
    logged_fragments: impl Iterator<Item = Result<PersistentFragmentLog, FragmentLogDeserializeError>>,
) -> Result<(Ledger, VecDeque<PersistentFragmentLog>), Error> {
    let block0_configuration = Block0Configuration::from_block(&block0).unwrap();

    let block0_start = block0_configuration.blockchain_configuration.block0_date;
    let slot_duration = block0_configuration.blockchain_configuration.slot_duration;

    let timeline = Timeline::new(timestamp_to_system_time(block0_start));

    let timeframe = TimeFrame::new(
        timeline,
        SlotDuration::from_secs(<u8>::from(slot_duration) as u32),
    );

    let mut ledger =
        Ledger::new(block0.header.id(), block0_fragments).map_err(Error::LedgerError)?;

    let parameters = ledger.get_ledger_parameters();

    let mut fragments: VecDeque<PersistentFragmentLog> = VecDeque::new();
    let mut tally_fragments: Vec<PersistentFragmentLog> = Vec::new();

    let mut vote_start = BlockDate::from_epoch_slot_id(999999998, 0);
    let mut vote_end = BlockDate::from_epoch_slot_id(999999999, 0);

    // process fragments lazily
    for fragment in logged_fragments {
        println!("{:?}", ledger.date());
        match fragment {
            Err(e) => {
                println!("Error processing fragment: {:?}", e);
            }
            Ok(fragment_log) => {
                let slot = timestamp_to_system_time(fragment_log.time);
                let new_slot = timeframe.slot_at(&slot).unwrap();
                let epoch_position = ledger.era().from_slot_to_era(new_slot).unwrap();
                let block_date = BlockDate::from(epoch_position);

                // discard votes that are not within the range
                if matches!(&fragment_log.fragment, Fragment::VoteCast(_)) {
                    if block_date >= vote_end || block_date < vote_start {
                        println!(
                            "Fragment {} with blockdate {} was discarded",
                            fragment_log.fragment.hash(),
                            block_date
                        );
                        continue;
                    }
                }

                if matches!(&fragment_log.fragment, Fragment::VoteTally(_)) {
                    tally_fragments.push(fragment_log);
                    continue;
                }

                match ledger.apply_fragment(&parameters, &fragment_log.fragment, ledger.date()) {
                    Ok(new_ledger) => {
                        ledger = new_ledger;
                        if matches!(&fragment_log.fragment, Fragment::VotePlan(_)) {
                            // TODO: for now we assume that voteplans use the same vote_start and vote_end
                            // data. We should check if those values actually changed and throw a proper error
                            let (vs, ve) = vote_start_date_from_ledger(&ledger);
                            vote_start = vs;
                            vote_end = ve;

                            ledger = ledger
                                .begin_block(
                                    parameters.clone(),
                                    ledger.chain_length().increase(),
                                    vote_start,
                                )
                                .unwrap()
                                .finish(&ConsensusEvalContext::Bft);
                        }
                    }
                    Err(e) => {
                        println!(
                            "Error processing fragment {}: {:?}",
                            &fragment_log.fragment.hash(),
                            e
                        ); // failed to apply fragment so store for later post-process
                        fragments.push_back(fragment_log);
                    }
                }
            }
        };
    }

    // postprocess failed fragments
    while !fragments.is_empty() {
        let len_before = fragments.len();

        fragments.retain(|fragment_log| {
            let slot = timestamp_to_system_time(fragment_log.time);
            let new_slot = timeframe.slot_at(&slot).unwrap();
            let epoch_position = ledger.era().from_slot_to_era(new_slot).unwrap();
            let block_date = BlockDate::from(epoch_position);

            // discard votes that are not within the range
            if matches!(&fragment_log.fragment, Fragment::VoteCast(_)) {
                if block_date >= vote_end || block_date < vote_start {
                    println!(
                        "Fragment {} with blockdate {} was discarded",
                        fragment_log.fragment.hash(),
                        block_date
                    );
                    return false;
                }
            }
            // take last added voteplan vote start date
            match ledger.apply_fragment(&parameters, &fragment_log.fragment, ledger.date()) {
                Ok(new_ledger) => {
                    ledger = new_ledger;
                    false
                }
                Err(e) => {
                    println!(
                        "Error processing fragment {}: {:?}",
                        &fragment_log.fragment.hash(),
                        e
                    );
                    true
                }
            }
        });

        if len_before == fragments.len() {
            break;
        }
    }

    // advance to tally time
    ledger = ledger
        .begin_block(
            parameters.clone(),
            ledger.chain_length().increase(),
            vote_end,
        )
        .unwrap()
        .finish(&ConsensusEvalContext::Bft);

    // run tally transactions
    for tally_fragment_log in tally_fragments {
        match ledger.apply_fragment(&parameters, &tally_fragment_log.fragment, ledger.date()) {
            Ok(new_ledger) => {
                ledger = new_ledger;
            }
            Err(e) => {
                println!(
                    "Error processing fragment {}: {:?}",
                    &tally_fragment_log.fragment.hash(),
                    e
                );
            }
        }
    }

    Ok((ledger, fragments))
}

#[cfg(test)]
mod test {
    use crate::cli::recovery::tally::mockchain::recover_ledger_from_fragments;
    use chain_impl_mockchain::block::Block;
    use chain_ser::deser::Deserialize;
    use jormungandr_lib::interfaces::{
        load_persistent_fragments_logs_from_folder_path, Block0Configuration,
    };
    use std::io::BufReader;
    use std::path::PathBuf;

    fn read_block0(path: PathBuf) -> std::io::Result<Block> {
        let reader = std::fs::File::open(path)?;
        Ok(Block::deserialize(BufReader::new(reader)).unwrap())
    }

    #[test]
    fn test_vote_flow() -> std::io::Result<()> {
        let path: PathBuf = r"D:\projects\rust\catalyst-toolbox\vote_flow_testing\fragment_logs"
            .parse()
            .unwrap();

        let fragments = load_persistent_fragments_logs_from_folder_path(&path)?;

        let block0 =
            read_block0(r"D:\projects\rust\catalyst-toolbox\vote_flow_testing\block-0.bin".into())?;

        let initial_fragments = block0.fragments();

        let (ledger, unprocessed) =
            recover_ledger_from_fragments(&block0, initial_fragments, fragments).unwrap();
        println!("{}", unprocessed.len());

        for voteplan in ledger.active_vote_plans() {
            for proposal in voteplan.proposals {
                println!("{:?}", proposal.tally);
            }
        }

        Ok(())
    }
}
