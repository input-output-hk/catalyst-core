use super::{encode, read_config, read_genesis_yaml, read_initials, write_genesis_yaml};
use crate::setup::start::QuickVitBackendSettingsBuilder;
use crate::Result;
use glob::glob;
use jormungandr_scenario_tests::ProgressBarMode as ScenarioProgressBarMode;
use jormungandr_scenario_tests::{Context, Seed};
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;
use vit_servicing_station_tests::common::data::ExternalValidVotingTemplateGenerator;

#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct PerfDataCommandArgs {
    /// Careful! directory would be removed before export
    #[structopt(long = "output", default_value = "./perf")]
    pub output_directory: PathBuf,

    /// configuration
    #[structopt(long = "config")]
    pub config: PathBuf,

    /// proposals import json
    #[structopt(
        long = "proposals",
        default_value = "../resources/external/proposals.json"
    )]
    pub proposals: PathBuf,

    /// challenges import json
    #[structopt(
        long = "challenges",
        default_value = "../resources/external/challenges.json"
    )]
    pub challenges: PathBuf,

    /// funds import json
    #[structopt(long = "funds", default_value = "../resources/external/funds.json")]
    pub funds: PathBuf,

    #[structopt(long = "snapshot")]
    pub snapshot: Option<PathBuf>,

    #[structopt(short = "p", long = "parts", default_value = "1")]
    pub parts: usize,

    #[structopt(short = "s", long = "single", default_value = "0")]
    pub single: usize,
}

impl PerfDataCommandArgs {
    pub fn exec(self) -> Result<()> {
        std::env::set_var("RUST_BACKTRACE", "full");

        let context = Context::new(
            Seed::generate(rand::rngs::OsRng),
            PathBuf::new(),
            PathBuf::new(),
            Some(self.output_directory.clone()),
            true,
            ScenarioProgressBarMode::None,
            "info".to_string(),
        );

        let mut quick_setup = QuickVitBackendSettingsBuilder::new();
        let config = read_config(&self.config)?;
        quick_setup.skip_qr_generation();
        quick_setup.upload_parameters(config.params.clone());
        quick_setup.fees(config.linear_fees);
        quick_setup.set_external_committees(config.committees);

        if !self.output_directory.exists() {
            std::fs::create_dir_all(&self.output_directory)?;
        }

        let title = quick_setup.title();
        let (vit_controller, mut controller, vit_parameters, version) =
            quick_setup.build(context)?;

        let mut template_generator = ExternalValidVotingTemplateGenerator::new(
            self.proposals.clone(),
            self.challenges.clone(),
            self.funds.clone(),
        )
        .unwrap();

        // generate vit station data
        let vit_station = vit_controller.spawn_vit_station(
            &mut controller,
            vit_parameters,
            &mut template_generator,
            version,
        )?;
        vit_station.shutdown();

        let mut root_directory = self.output_directory.clone();
        root_directory.push(title);

        let mut genesis = root_directory.clone();
        genesis.push("genesis.yaml");

        let mut block0 = root_directory.clone();
        block0.push("block0.bin");

        let mut block0_configuration = read_genesis_yaml(&genesis)?;

        if !config.consensus_leader_ids.is_empty() {
            block0_configuration
                .blockchain_configuration
                .consensus_leader_ids = config.consensus_leader_ids;
        }

        if let Some(snapshot_file) = &self.snapshot {
            let snapshot = read_initials(&snapshot_file)?;
            block0_configuration.initial.extend(snapshot);
        }
        let mut single_directory = root_directory.clone();
        single_directory.push("single");
        self.move_single_user_secrets(&root_directory, &single_directory)?;
        self.split_secrets(root_directory)?;

        write_genesis_yaml(block0_configuration, &genesis)?;
        println!("genesis.yaml: {:?}", std::fs::canonicalize(&genesis)?);
        encode(&genesis, &block0)?;
        println!("block0: {:?}", std::fs::canonicalize(&block0)?);
        println!("Fund id: {}", quick_setup.parameters().fund_id);
        println!(
            "voteplan ids: {:?}",
            controller
                .vote_plans()
                .iter()
                .map(|x| x.id())
                .collect::<Vec<String>>()
        );
        println!(
            "vote start timestamp: {:?}",
            quick_setup.parameters().vote_start_timestamp
        );
        println!(
            "tally start timestamp: {:?}",
            quick_setup.parameters().tally_start_timestamp
        );
        println!(
            "tally end timestamp: {:?}",
            quick_setup.parameters().tally_end_timestamp
        );
        println!(
            "next vote start time: {:?}",
            quick_setup.parameters().next_vote_start_time
        );
        println!(
            "refresh timestamp: {:?}",
            quick_setup.parameters().refresh_time
        );
        Ok(())
    }

    fn move_single_user_secrets<P: AsRef<Path>>(&self, root: P, output_folder: P) -> Result<()> {
        let pattern = format!("{}/wallet_*_*", root.as_ref().display());
        for file in glob(&pattern)
            .expect("Failed to read glob pattern")
            .take(self.single)
        {
            let file = file?;
            let file_name = file.file_name().unwrap();
            std::fs::rename(file.clone(), output_folder.as_ref().join(file_name))?;
        }
        Ok(())
    }

    fn split_secrets<P: AsRef<Path>>(&self, root: P) -> Result<()> {
        let pattern = format!("{}/wallet_*_*", root.as_ref().display());

        let secrets: Vec<PathBuf> = (0..self.parts)
            .into_iter()
            .map(|id| {
                let folder = root.as_ref().join("secrets".to_owned() + &id.to_string());
                std::fs::create_dir_all(&folder).unwrap();
                folder
            })
            .collect();

        for (id, file) in glob(&pattern)
            .expect("Failed to read glob pattern")
            .enumerate()
        {
            let folder = &secrets[id % secrets.len()];
            let file = file?;
            let file_name = file.file_name().unwrap();
            std::fs::rename(file.clone(), folder.join(file_name))?;
        }
        Ok(())
    }
}
