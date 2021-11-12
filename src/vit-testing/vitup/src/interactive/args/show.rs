use crate::interactive::VitInteractiveCommandExec;
use chain_impl_mockchain::block::BlockDate;
use chrono::NaiveDateTime;
use chrono::Utc;
use jormungandr_lib::interfaces::BlockchainConfiguration;
use jormungandr_scenario_tests::interactive::args::show::ShowStatus as BasicShowStatus;
use jormungandr_scenario_tests::test::Result;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum Show {
    /// Prints which nodes are upp
    Status(ShowStatus),
    /// Prints fragments counts
    FragmentCount(jormungandr_scenario_tests::interactive::args::show::ShowFragmentCount),
    /// Prints received fragment list
    Fragments(jormungandr_scenario_tests::interactive::args::show::ShowFragments),
    /// Prints block height
    BlockHeight(jormungandr_scenario_tests::interactive::args::show::ShowBlockHeight),
    /// Prints peers stats
    PeerStats(jormungandr_scenario_tests::interactive::args::show::ShowPeerStats),
    /// Prints stats
    Stats(jormungandr_scenario_tests::interactive::args::show::ShowNodeStats),
    /// Prints logs, can filter logs to print
    /// only errors or filter by custom string  
    Logs(jormungandr_scenario_tests::interactive::args::show::ShowLogs),
    /// Active Vote Plans
    VotePlans(jormungandr_scenario_tests::interactive::args::show::ActiveVotePlans),
    /// Vote timing
    VoteTime(VoteTimeStatus),
}

impl Show {
    pub fn exec(&self, command: &mut VitInteractiveCommandExec) -> Result<()> {
        match self {
            Show::Status(status) => status.exec(command)?,
            Show::Stats(stats) => stats.exec(command.controller_mut()),
            Show::FragmentCount(fragment_counts) => fragment_counts.exec(command.controller_mut()),
            Show::Fragments(fragments) => fragments.exec(command.controller_mut()),
            Show::BlockHeight(block_height) => block_height.exec(command.controller_mut()),
            Show::PeerStats(peer_stats) => peer_stats.exec(command.controller_mut()),
            Show::Logs(logs) => logs.exec(command.controller_mut()),
            Show::VotePlans(active_vote_plan) => active_vote_plan.exec(command.controller_mut()),
            Show::VoteTime(vote_status) => vote_status.exec(command)?,
        };
        Ok(())
    }
}

#[derive(StructOpt, Debug)]
pub struct ShowStatus {
    #[structopt(short = "a", long = "alias")]
    pub alias: Option<String>,
}

impl ShowStatus {
    pub fn exec(&self, command: &mut VitInteractiveCommandExec) -> Result<()> {
        let basic_show_status = BasicShowStatus {
            alias: self.alias.clone(),
        };

        basic_show_status.exec(command.controller_mut());

        for vit_station in command.vit_controller.vit_stations() {
            println!("{} is up", vit_station.alias());
        }

        for proxy_wallet in command.vit_controller.proxies() {
            println!("{} is up", proxy_wallet.alias());
        }
        Ok(())
    }
}

#[derive(StructOpt, Debug)]
pub struct VoteTimeStatus {
    #[structopt(short = "a", long = "alias")]
    pub alias: Option<String>,
}

impl VoteTimeStatus {
    pub fn exec(&self, command: &mut VitInteractiveCommandExec) -> Result<()> {
        let controller = &command.controller_mut();

        let blockchain_configuration = &controller
            .controller()
            .settings()
            .block0
            .blockchain_configuration;
        let node = controller.nodes().iter().next().unwrap();
        let vote_plans = node.rest().vote_plan_statuses().unwrap();
        let vote_plan = vote_plans.first().unwrap();

        let mut dates = vec![
            (
                "Voting period start",
                self.calculate_date(blockchain_configuration, vote_plan.vote_start.into()),
            ),
            (
                "Voting period end",
                self.calculate_date(blockchain_configuration, vote_plan.vote_end.into()),
            ),
            (
                "Tally period end",
                self.calculate_date(blockchain_configuration, vote_plan.committee_end.into()),
            ),
            ("> Current time", Utc::now().naive_utc()),
        ];

        dates.sort_by(|a, b| a.1.cmp(&b.1));

        println!("======================================");
        println!(
            "Blockchain start: {}",
            NaiveDateTime::from_timestamp(blockchain_configuration.block0_date.to_secs() as i64, 0)
        );
        for (alias, date) in dates {
            println!("{}: {}", alias, date);
        }
        println!("======================================");
        Ok(())
    }

    pub fn calculate_date(
        &self,
        blockchain_configuration: &BlockchainConfiguration,
        block_date: BlockDate,
    ) -> NaiveDateTime {
        let slot_duration: u8 = blockchain_configuration.slot_duration.into();
        let slots_per_epoch: u32 = blockchain_configuration.slots_per_epoch.into();
        let epoch_duration: u64 = (slot_duration as u64) * (slots_per_epoch as u64);

        let block0_date_secs = blockchain_configuration.block0_date.to_secs() as i64;

        let block_epoch_part = epoch_duration as i64 * block_date.epoch as i64;
        let block_slot_part = slot_duration as i64 * block_date.slot_id as i64;

        let timestamp = block0_date_secs + block_epoch_part + block_slot_part;

        NaiveDateTime::from_timestamp(timestamp, 0)
    }
}
