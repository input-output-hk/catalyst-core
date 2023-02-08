mod blockchain;
mod builder;
mod initials;
mod migrations;
mod service;
mod static_data;
mod vote_plan;
mod vote_time;

mod additional;
pub mod certs;
pub mod mode;

use crate::config::additional::AdditionalServices;
use crate::config::builder::convert_to_human_date;
use crate::config::vote_time::FORMAT;
use crate::mode::standard::VitController;
use crate::Result;
pub use blockchain::Blockchain;
pub use builder::ConfigBuilder;
pub use certs::CertificatesBuilder;
use chain_addr::Discrimination;
pub use initials::{
    Block0Initial, Block0Initials, Initials, Role, DIRECT_VOTING_GROUP, REP_VOTING_GROUP,
};
pub use migrations::{Error as MigrationError, MigrationFilesBuilder};
use serde::{Deserialize, Serialize};
pub use service::Service;
use snapshot_lib::VoterHIR;
pub use static_data::StaticData;
use std::fmt::Debug;
use std::path::Path;
use std::time::Duration;
use time::format_description::{self, FormatItem};
use tracing::{span, Level};
use valgrind::Protocol;
pub use vote_plan::VotePlan;
pub use vote_time::{VoteBlockchainTime, VoteTime, FORMAT as VOTE_TIME_FORMAT};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub initials: Initials,
    #[serde(default)]
    pub vote_plan: VotePlan,
    #[serde(default)]
    pub blockchain: Blockchain,
    #[serde(default)]
    pub data: StaticData,
    #[serde(default)]
    pub service: Service,
    #[serde(default)]
    pub additional: AdditionalServices,
}

impl Config {
    pub fn protocol<P: AsRef<Path>>(&self, working_dir: P) -> Result<Protocol> {
        if self.service.https {
            Ok(CertificatesBuilder::default().build(working_dir)?.into())
        } else {
            Ok(Default::default())
        }
    }

    pub fn extend_from_initials_file<P: AsRef<Path>>(
        &mut self,
        snapshot: P,
        discrimination: Discrimination,
    ) -> Result<()> {
        let snapshot = snapshot.as_ref();
        if !snapshot.exists() {
            return Err(crate::error::Error::CannotFindSnapshotFile(
                snapshot.to_path_buf(),
            ));
        }
        self.initials
            .block0
            .extend_from_external(read_voter_hirs(snapshot)?, discrimination);
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

    pub fn print_report(&self, controller: Option<VitController>) {
        let (vote_start_timestamp, tally_start_timestamp, tally_end_timestamp) =
            convert_to_human_date(self);

        let span = span!(Level::INFO, "deployment report");
        let _enter = span.enter();

        if let Some(controller) = controller {
            tracing::info!(
                "voteplan ids: \t\t{:?}",
                controller
                    .defined_vote_plans()
                    .iter()
                    .map(|x| x.id())
                    .collect::<Vec<String>>()
            );
        }

        tracing::info!(
            "Fund id: \t\t\t{}",
            self.data.current_fund.fund_info.fund_id
        );
        tracing::info!(
            "block0 date:\t\t(block0_date):\t\t\t\t\t{}",
            jormungandr_lib::time::SystemTime::from_secs_since_epoch(
                self.blockchain.block0_date_as_unix().to_secs()
            )
        );

        tracing::info!(
            "refresh timestamp:\t\t(registration_snapshot_time):\t\t\t{}",
            self.data.current_fund.dates.snapshot_time
        );

        tracing::info!(
            "vote start timestamp:\t(fund_start_time, chain_vote_start_time):\t{}",
            vote_start_timestamp
        );
        tracing::info!(
            "tally start timestamp:\t(fund_end_time, chain_vote_end_time):\t\t{}",
            tally_start_timestamp
        );
        tracing::info!(
            "tally end timestamp:\t(chain_committee_end_time):\t\t\t{}",
            tally_end_timestamp
        );
        tracing::info!(
            "next refresh timestamp:\t(next registration_snapshot_time):\t\t{}",
            self.data.current_fund.dates.next_snapshot_time
        );
        tracing::info!(
            "next vote start time:\t(next_fund_start_time):\t\t\t\t{}",
            self.data.current_fund.dates.next_vote_start_time
        );
    }
}

#[tracing::instrument]
pub fn read_config<P: AsRef<Path> + Debug>(config: P) -> Result<Config> {
    let config = config.as_ref();
    if !config.exists() {
        return Err(crate::error::Error::CannotFindConfig(config.to_path_buf()));
    }

    let contents = std::fs::read_to_string(config)?;
    serde_json::from_str(&contents).map_err(Into::into)
}

pub fn date_format() -> Vec<FormatItem<'static>> {
    format_description::parse(FORMAT).unwrap()
}

pub fn read_voter_hirs<P: AsRef<Path>>(snapshot: P) -> Result<Vec<VoterHIR>> {
    let snapshot = snapshot.as_ref();
    if !snapshot.exists() {
        return Err(crate::error::Error::CannotFindSnapshotFile(
            snapshot.to_path_buf(),
        ));
    }

    let contents = std::fs::read_to_string(snapshot)?;
    serde_json::from_str(&contents).map_err(Into::into)
}
