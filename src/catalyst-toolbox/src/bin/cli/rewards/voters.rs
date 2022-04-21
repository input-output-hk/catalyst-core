use super::Error;
use catalyst_toolbox::rewards::voters::{calc_voter_rewards, Rewards, VoteCount};
use catalyst_toolbox::snapshot::{registration::MainnetRewardAddress, Snapshot};
use catalyst_toolbox::utils::assert_are_close;

use jcli_lib::jcli_lib::block::Common;
use jormungandr_lib::interfaces::Block0Configuration;

use structopt::StructOpt;

use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct VotersRewards {
    #[structopt(flatten)]
    common: Common,
    /// Reward (in LOVELACE) to be distributed
    #[structopt(long)]
    total_rewards: u64,

    /// Path to raw snapshot
    #[structopt(long)]
    snapshot_path: PathBuf,

    /// Stake threshold to be able to participate in a Catalyst sidechain
    /// Registrations with less than the threshold associated to the stake address
    /// will be ignored
    #[structopt(long)]
    registration_threshold: u64,

    #[structopt(long)]
    votes_count_path: PathBuf,

    /// Number of votes required to be able to receive voter rewards
    #[structopt(long, default_value)]
    vote_threshold: u64,
}

fn write_rewards_results(
    common: Common,
    rewards: &BTreeMap<MainnetRewardAddress, Rewards>,
) -> Result<(), Error> {
    let writer = common.open_output()?;
    let header = ["Address", "Reward for the voter (lovelace)"];
    let mut csv_writer = csv::Writer::from_writer(writer);
    csv_writer.write_record(&header).map_err(Error::Csv)?;

    for (address, rewards) in rewards.iter() {
        let record = [address.to_string(), rewards.trunc().to_string()];
        csv_writer.write_record(&record).map_err(Error::Csv)?;
    }
    Ok(())
}

impl VotersRewards {
    pub fn exec(self) -> Result<(), Error> {
        let VotersRewards {
            common,
            total_rewards,
            snapshot_path,
            registration_threshold,
            votes_count_path,
            vote_threshold,
        } = self;
        let block = common.input.load_block()?;
        let block0 = Block0Configuration::from_block(&block)
            .map_err(jcli_lib::jcli_lib::block::Error::BuildingGenesisFromBlock0Failed)?;

        let vote_count: VoteCount = serde_json::from_reader(jcli_lib::utils::io::open_file_read(
            &Some(votes_count_path),
        )?)?;

        let snapshot = Snapshot::from_raw_snapshot(
            serde_json::from_reader(jcli_lib::utils::io::open_file_read(&Some(snapshot_path))?)?,
            registration_threshold.into(),
        );

        let results = calc_voter_rewards(
            vote_count,
            vote_threshold,
            &block0,
            snapshot,
            Rewards::from(total_rewards),
        )?;

        let actual_rewards = results.values().sum::<Rewards>();
        assert_are_close(actual_rewards, Rewards::from(total_rewards));

        write_rewards_results(common, &results)?;
        Ok(())
    }
}
