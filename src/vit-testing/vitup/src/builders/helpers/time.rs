use crate::config::{Config, VoteBlockchainTime, VoteTime};
use chrono::NaiveDateTime;
use chrono::Utc;

pub fn convert_to_blockchain_date(config: &Config) -> VoteBlockchainTime {
    match config.vote_plan.vote_time {
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

pub fn convert_to_human_date(config: &Config) -> (NaiveDateTime, NaiveDateTime, NaiveDateTime) {
    let config = config.clone();

    match config.vote_plan.vote_time {
        VoteTime::Blockchain(blockchain) => {
            let block0_date = config.blockchain.block0_date_as_unix();

            let epoch_duration =
                config.blockchain.slot_duration as u32 * blockchain.slots_per_epoch;
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

pub fn default_snapshot_date() -> NaiveDateTime {
    let dt = Utc::now();
    NaiveDateTime::from_timestamp((dt - chrono::Duration::hours(3)).timestamp(), 0)
}

pub fn default_next_vote_date() -> NaiveDateTime {
    let dt = Utc::now();
    NaiveDateTime::from_timestamp((dt + chrono::Duration::days(30)).timestamp(), 0)
}

pub fn default_next_snapshot_date() -> NaiveDateTime {
    let dt = Utc::now();
    NaiveDateTime::from_timestamp((dt + chrono::Duration::days(29)).timestamp(), 0)
}

/*
TODO uncomment once implementation is provided

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_test() {
        let block0_date = SecondsSinceUnixEpoch::now();
        let mut parameters = Config::default();

        let vote_time = VoteTime::real_from_str(
            "2021-10-06 11:00:00",
            "2021-10-06 18:00:00",
            "2021-10-07 09:00:00",
        )
        .unwrap();

        parameters.slot_duration = 10;
        parameters.vote_time = vote_time.clone();
        println!("Before {:#?}", vote_time);
        let blockchain_time = convert_to_blockchain_date(&parameters, block0_date);
        parameters.vote_time = VoteTime::Blockchain(blockchain_time);
        println!(
            "After {:#?}",
            convert_to_human_date(&parameters, block0_date)
        );
    }
}
*/
