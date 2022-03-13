mod blockchain;
mod builder;
mod initials;
mod service;
mod static_data;
mod vote_plan;
mod vote_time;

pub mod certs;
pub mod mode;

use crate::config::builder::convert_to_human_date;
use crate::config::vote_time::FORMAT;
pub use blockchain::Blockchain;
pub use builder::ConfigBuilder;
pub use certs::CertificatesBuilder;
pub use initials::{Initial as InitialEntry, Initials};
pub use service::Service;
pub use static_data::StaticData;
use time::format_description::{self, FormatItem};
use valgrind::Protocol;
pub use vote_plan::VotePlan;
pub use vote_time::{VoteBlockchainTime, VoteTime, FORMAT as VOTE_TIME_FORMAT};

use crate::Result;
use jormungandr_automation::testing::block0::read_initials;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub initials: Initials,
    #[serde(flatten)]
    pub vote_plan: VotePlan,
    #[serde(default)]
    pub blockchain: Blockchain,
    #[serde(default)]
    pub data: StaticData,
    #[serde(default)]
    pub service: Service,
}

impl Config {
    pub fn protocol<P: AsRef<Path>>(&self, working_dir: P) -> Result<Protocol> {
        if self.service.https {
            Ok(CertificatesBuilder::default().build(working_dir)?.into())
        } else {
            Ok(Default::default())
        }
    }

    pub fn extend_from_initials_file<P: AsRef<Path>>(&mut self, snapshot: P) -> Result<()> {
        self.initials.extend_from_external(read_initials(snapshot)?);
        Ok(())
    }

    pub fn calculate_vote_duration(&self) -> Duration {
        match self.vote_plan.vote_time {
            VoteTime::Blockchain(blockchain) => {
                let duration_as_secs = (blockchain.tally_end - blockchain.vote_start) as u64
                    * self.blockchain.slot_duration as u64
                    * (blockchain.slots_per_epoch - 1) as u64;

                Duration::from_secs(duration_as_secs)
            }
            VoteTime::Real {
                vote_start_timestamp,
                tally_start_timestamp,
                tally_end_timestamp: _,
                find_best_match: _,
            } => Duration::from_secs(
                (tally_start_timestamp - vote_start_timestamp).whole_seconds() as u64,
            ),
        }
    }

    pub fn print_report(&self) {
        let (vote_start_timestamp, tally_start_timestamp, tally_end_timestamp) =
            convert_to_human_date(self);

        println!("Fund id: {}", self.data.fund_id);
        println!(
            "block0 date:\t\t(block0_date):\t\t\t\t\t{}",
            jormungandr_lib::time::SystemTime::from_secs_since_epoch(
                self.blockchain.block0_date_as_unix().to_secs()
            )
        );

        println!(
            "refresh timestamp:\t(registration_snapshot_time):\t\t\t{:?}",
            self.data.snapshot_time
        );

        println!(
            "vote start timestamp:\t(fund_start_time, chain_vote_start_time):\t{:?}",
            vote_start_timestamp
        );
        println!(
            "tally start timestamp:\t(fund_end_time, chain_vote_end_time):\t\t{:?}",
            tally_start_timestamp
        );
        println!(
            "tally end timestamp:\t(chain_committee_end_time):\t\t\t{:?}",
            tally_end_timestamp
        );
        println!(
            "next refresh timestamp:\t(next registration_snapshot_time):\t\t{:?}",
            self.data.next_snapshot_time
        );
        println!(
            "next vote start time:\t(next_fund_start_time):\t\t\t\t{:?}",
            self.data.next_vote_start_time
        );
    }
}

pub fn read_config<P: AsRef<Path>>(config: P) -> Result<Config> {
    let contents = std::fs::read_to_string(&config)?;
    serde_json::from_str(&contents).map_err(Into::into)
}

pub fn date_format() -> Vec<FormatItem<'static>> {
    format_description::parse(FORMAT).unwrap()
}
