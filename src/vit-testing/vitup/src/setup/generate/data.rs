use crate::config::DataGenerationConfig;
use crate::setup::start::QuickVitBackendSettingsBuilder;
use crate::Result;
use chain_core::property::Block;
use chain_core::property::Serialize;
use chain_impl_mockchain::ledger::Ledger;
use jormungandr_lib::interfaces::Block0Configuration;
use jormungandr_scenario_tests::ProgressBarMode as ScenarioProgressBarMode;
use jormungandr_scenario_tests::{Context, Seed};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct DataCommandArgs {
    /// Careful! directory would be removed before export
    #[structopt(long = "output", default_value = "./data")]
    pub output_directory: PathBuf,

    /// how many qr to generate
    #[structopt(long = "config")]
    pub config: PathBuf,
}

pub fn read_config<P: AsRef<Path>>(config: P) -> Result<DataGenerationConfig> {
    let contents = std::fs::read_to_string(&config)?;
    serde_json::from_str(&contents).map_err(Into::into)
}

pub fn read_genesis_yaml<P: AsRef<Path>>(genesis: P) -> Result<Block0Configuration> {
    let contents = std::fs::read_to_string(&genesis)?;
    serde_yaml::from_str(&contents).map_err(Into::into)
}

pub fn write_genesis_yaml<P: AsRef<Path>>(genesis: Block0Configuration, path: P) -> Result<()> {
    use std::fs::OpenOptions;
    use std::io::{prelude::*, Seek, SeekFrom};

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)
        .unwrap();

    let content = serde_yaml::to_string(&genesis)?;

    file.seek(SeekFrom::Start(0))?;
    file.write_all(&content.as_bytes())?;
    Ok(())
}

pub fn encode<P: AsRef<Path>, Q: AsRef<Path>>(genesis: P, block0: Q) -> Result<()> {
    let input: std::fs::File = std::fs::OpenOptions::new()
        .create(false)
        .write(false)
        .read(true)
        .append(false)
        .truncate(false)
        .open(&genesis)?;

    let output: std::fs::File = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .read(false)
        .append(false)
        .truncate(true)
        .open(&block0)?;

    let genesis: Block0Configuration = serde_yaml::from_reader(input)?;
    let block = genesis.to_block();
    Ledger::new(block.id(), block.fragments())?;
    block.serialize(&output).map_err(Into::into)
}

impl DataCommandArgs {
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

        quick_setup.upload_parameters(config.params.clone());

        if !self.output_directory.exists() {
            std::fs::create_dir_all(&self.output_directory)?;
        }

        let title = quick_setup.title();
        let (vit_controller, mut controller, vit_parameters) = quick_setup.build(context)?;

        // generate vit station data
        let vit_station = vit_controller.spawn_vit_station(&mut controller, vit_parameters)?;
        vit_station.shutdown();

        let mut root_directory = self.output_directory;
        root_directory.push(title);

        let mut genesis = root_directory.clone();
        genesis.push("genesis.yaml");

        let mut block0 = root_directory;
        block0.push("block0.bin");

        let mut block0_configuration = read_genesis_yaml(&genesis)?;

        block0_configuration.blockchain_configuration.linear_fees = config.linear_fees;
        if !config.consensus_leader_ids.is_empty() {
            block0_configuration
                .blockchain_configuration
                .consensus_leader_ids = config.consensus_leader_ids;
        }
        if !config.committees.is_empty() {
            block0_configuration
                .blockchain_configuration
                .committees
                .extend(config.committees.clone());
        }
        if !config.additions.is_empty() {
            block0_configuration.initial.extend(config.additions);
        }

        println!("{:?}",block0_configuration);

        write_genesis_yaml(block0_configuration, &genesis)?;
        encode(&genesis, &block0)
    }
}
