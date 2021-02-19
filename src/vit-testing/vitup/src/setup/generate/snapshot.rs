use crate::config::Initials;
use crate::setup::generate::data::read_genesis_yaml;
use crate::setup::start::QuickVitBackendSettingsBuilder;
use crate::Result;
use jormungandr_scenario_tests::ProgressBarMode as ScenarioProgressBarMode;
use jormungandr_scenario_tests::{Context, Seed};
use jortestkit::prelude::read_file;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;

use jormungandr_lib::interfaces::Initial;

#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct SnapshotCommandArgs {
    /// Careful! directory would be removed before export
    #[structopt(long = "root-dir", default_value = "./data")]
    pub output_directory: PathBuf,

    /// how many addresses to generate
    #[structopt(long = "count")]
    pub initials: Option<usize>,

    #[structopt(long = "initials")]
    pub initials_mapping: Option<PathBuf>,

    #[structopt(long = "global-pin", default_value = "1234")]
    pub global_pin: String,
}

impl SnapshotCommandArgs {
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

        if let Some(mapping) = self.initials_mapping {
            let content = read_file(mapping);
            let initials: Initials =
                serde_json::from_str(&content).expect("JSON was not well-formatted");
            quick_setup.initials(initials);
        } else if let Some(initials_count) = self.initials {
            quick_setup.initials_count(initials_count, &self.global_pin);
        }

        if !self.output_directory.exists() {
            std::fs::create_dir_all(&self.output_directory)?;
        } else {
            std::fs::remove_dir_all(&self.output_directory)?;
        }

        let (_, controller, _) = quick_setup.build(context)?;

        let result_dir = std::path::Path::new(&self.output_directory).join(quick_setup.title());
        let genesis_yaml = std::path::Path::new(&result_dir).join("genesis.yaml");

        //remove all files except qr codes and genesis
        for entry in std::fs::read_dir(result_dir.clone())? {
            let entry = entry?;
            let md = std::fs::metadata(entry.path()).unwrap();
            if md.is_dir() {
                continue;
            }

            if entry.path() == genesis_yaml {
                continue;
            }

            std::fs::remove_file(entry.path())?;
        }

        //rename qr codes to {address}_{pin}.png syntax
        let qr_codes = Path::new(&result_dir).join("qr-codes");
        for entry in std::fs::read_dir(&qr_codes)? {
            let entry = entry?;
            let path = entry.path();

            let file_name = path
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .replace(&format!("_{}", self.global_pin), "");

            let wallet = controller.wallet(&file_name)?;
            let new_file_name = format!("{}_{}.png", wallet.address(), self.global_pin);
            std::fs::rename(
                path.clone(),
                std::path::Path::new(path.parent().unwrap()).join(&new_file_name),
            )?;
        }

        // write snapshot.json
        let config = read_genesis_yaml(genesis_yaml.clone())?;

        let initials: Vec<Initial> = config
            .initial
            .iter()
            .filter(|x| matches!(x, Initial::Fund { .. }))
            .cloned()
            .collect();

        let snapshot = Snapshot { initial: initials };
        let snapshot_ser = serde_json::to_string_pretty(&snapshot)?;

        let mut file = std::fs::File::create(Path::new(&result_dir).join("snapshot.json"))?;
        file.write_all(snapshot_ser.as_bytes())?;

        std::fs::remove_file(genesis_yaml)?;

        println!("Snapshot dumped into {:?}", file);
        println!("Qr codes dumped into {:?}", qr_codes);
       
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct Snapshot {
    pub initial: Vec<Initial>,
}
