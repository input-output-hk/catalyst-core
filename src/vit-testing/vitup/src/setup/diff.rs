use crate::Result;
use assert_fs::TempDir;
use chain_core::property::Deserialize;
use chain_impl_mockchain::block::Block;
use diffy::create_patch;
use diffy::PatchFormatter;
use iapyx::WalletBackend;
use jormungandr_lib::interfaces::Block0Configuration;
use jortestkit::prelude::read_file;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use structopt::StructOpt;
use vit_servicing_station_tests::common::startup::server::ServerBootstrapper;

#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct DiffCommand {
    #[structopt(short = "l", long = "local", default_value = "./data")]
    pub local: PathBuf,

    #[structopt(short = "t", long = "target")]
    pub target: String,
}

impl DiffCommand {
    pub fn exec(self) -> Result<()> {
        std::env::set_var("RUST_BACKTRACE", "full");
        let remote = TempDir::new().unwrap();

        let local_genesis_yaml = Path::new(&self.local).join("genesis.yaml");
        let local_vit_db = Path::new(&self.local).join("vit_station/storage.db");
        let remote_client = WalletBackend::new(self.target, Default::default());

        let remote_genesis_yaml = remote.path().join("genesis_remote.yaml");

        decode_block0(remote_client.block0()?, remote_genesis_yaml.clone())?;

        let local_genesis_content = read_file(local_genesis_yaml);
        let remote_genesis_content = read_file(remote_genesis_yaml);

        let server = ServerBootstrapper::new()
            .with_db_path(local_vit_db.to_str().unwrap())
            .start(&remote)?;
        //       .start_with_exe(&remote, PathBuf::from_str("vit-servicing-station-server"))?;

        let local_funds = server.rest_client().funds()?;
        let remote_funds = remote_client.funds()?;

        let patch = create_patch(&remote_genesis_content, &local_genesis_content);
        let f = PatchFormatter::new().with_color();
        println!("**** GENESIS YAML DIFF ****");
        print!("{}", f.fmt_patch(&patch));
        Ok(())
    }
}

fn decode_block0<Q: AsRef<Path>>(block0: Vec<u8>, genesis_yaml: Q) -> Result<()> {
    let writer: std::fs::File = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .read(false)
        .append(false)
        .truncate(true)
        .open(&genesis_yaml)?;

    let yaml = Block0Configuration::from_block(&Block::deserialize(&*block0)?)?;
    Ok(serde_yaml::to_writer(writer, &yaml)?)
}
