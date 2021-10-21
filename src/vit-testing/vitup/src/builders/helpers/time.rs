use crate::config::VitStartParameters;
use crate::config::VoteBlockchainTime;
use crate::config::VoteTime;
use chrono::NaiveDateTime;
use jormungandr_lib::time::SecondsSinceUnixEpoch;

pub fn convert_to_blockchain_date(
    parameters: &VitStartParameters,
    _block0_date: SecondsSinceUnixEpoch,
) -> VoteBlockchainTime {
    match parameters.vote_time {
        VoteTime::Blockchain(blockchain_date) => blockchain_date,
        // TODO Implement proper conversion.
        // Right now it's not used.
        VoteTime::Real {
            vote_start_timestamp: _,
            tally_start_timestamp: _,
            tally_end_timestamp: _,
            find_best_match: _,
        } => {
            unimplemented!()
        }
    }
}

pub fn convert_to_human_date(
    parameters: &VitStartParameters,
    block0_date: SecondsSinceUnixEpoch,
) -> (NaiveDateTime, NaiveDateTime, NaiveDateTime) {
    let parameters = parameters.clone();

    println!(
        "Current date {:?}",
        NaiveDateTime::from_timestamp(block0_date.to_secs() as i64, 0)
    );

    match parameters.vote_time {
        VoteTime::Blockchain(blockchain) => {
            let epoch_duration = parameters.slot_duration as u32 * blockchain.slots_per_epoch;
            let vote_start_timestamp =
                block0_date.to_secs() as u32 + epoch_duration * blockchain.vote_start;
            let vote_start_timestamp =
                NaiveDateTime::from_timestamp(vote_start_timestamp as i64, 0);
            let tally_start_timestamp =
                block0_date.to_secs() as u32 + epoch_duration * blockchain.tally_start;
            let tally_start_timestamp =
                NaiveDateTime::from_timestamp(tally_start_timestamp as i64, 0);
            let tally_end_timestamp =
                block0_date.to_secs() as u32 + epoch_duration * blockchain.tally_end;
            let tally_end_timestamp = NaiveDateTime::from_timestamp(tally_end_timestamp as i64, 0);

            (
                vote_start_timestamp,
                tally_start_timestamp,
                tally_end_timestamp,
            )
        }
        VoteTime::Real {
            vote_start_timestamp,
            tally_start_timestamp,
            tally_end_timestamp,
            find_best_match: _,
        } => (
            vote_start_timestamp,
            tally_start_timestamp,
            tally_end_timestamp,
        ),
    }
}
