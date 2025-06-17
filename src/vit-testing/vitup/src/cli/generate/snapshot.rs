use crate::builders::utils::SessionSettingsExtension;
use crate::builders::utils::{logger, DeploymentTree};
use crate::builders::VitBackendSettingsBuilder;
use crate::config::Block0Initials;
use crate::config::ConfigBuilder;
use crate::Result;
use clap::Parser;
use hersir::config::SessionSettings;
use jormungandr_automation::jormungandr::LogLevel;
use jormungandr_automation::testing::block0::read_genesis_yaml;
use jormungandr_lib::interfaces::Initial;
use jortestkit::prelude::read_file;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct SnapshotCommandArgs {
    /// Careful! directory would be removed before export
    #[clap(long = "root-dir", default_value = "./data")]
    pub output_directory: PathBuf,

    /// how many addresses to generate
    #[clap(long = "count")]
    pub initials: Option<usize>,

    #[clap(long = "initials", conflicts_with = "count")]
    pub initials_mapping: Option<PathBuf>,

    #[clap(long = "global-pin", default_value = "1234")]
    pub global_pin: String,

    #[clap(long = "skip-qr-generation")]
    pub skip_qr_generation: bool,

    #[clap(long = "log-level", default_value = "LogLevel::INFO")]
    pub log_level: LogLevel,
}

impl SnapshotCommandArgs {
    pub fn exec(self) -> Result<()> {
        std::env::set_var("RUST_BACKTRACE", "full");

        logger::init(self.log_level)?;

        let session_settings = SessionSettings::from_dir(&self.output_directory);

        let mut config_builder = ConfigBuilder::default();

        if let Some(mapping) = self.initials_mapping {
            let content = read_file(mapping)?;
            let initials: Block0Initials =
                serde_json::from_str(&content).expect("JSON was not well-formatted");
            config_builder = config_builder.block0_initials(initials);
        } else {
            config_builder =
                config_builder.block0_initials_count(self.initials.unwrap(), &self.global_pin);
        }

        let mut quick_setup = VitBackendSettingsBuilder::default();

        if self.skip_qr_generation {
            quick_setup = quick_setup.skip_qr_generation();
        }

        if !self.output_directory.exists() {
            std::fs::create_dir_all(&self.output_directory)?;
        } else {
            std::fs::remove_dir_all(&self.output_directory)?;
        }

        let deployment_tree = DeploymentTree::new(&self.output_directory);

        let (mut controller, _) = quick_setup
            .session_settings(session_settings)
            .config(&config_builder.build())
            .build()?;

        let genesis_yaml = deployment_tree.genesis_path();

        //remove all files except qr codes and genesis
        for entry in std::fs::read_dir(deployment_tree.root_path())? {
            let entry = entry?;
            let md = std::fs::metadata(entry.path()).unwrap();
            if md.is_dir() {
                continue;
            }

            if entry.path() == genesis_yaml {
                continue;
            }

            //skip secret key generation
            if entry.file_name().to_str().unwrap().contains("wallet") {
                continue;
            }

            std::fs::remove_file(entry.path())?;
        }

        if !self.skip_qr_generation {
            //rename qr codes to {address}_{pin}.png syntax
            let qr_codes = deployment_tree.qr_codes_path();
            let mut i = 1;
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
                let new_file_name = format!("{}_{}_{}.png", i, wallet.address(), self.global_pin);
                i += 1;
                std::fs::rename(
                    path.clone(),
                    std::path::Path::new(path.parent().unwrap()).join(new_file_name),
                )?;
            }
            println!("Qr codes dumped into {:?}", qr_codes);
        }

        // write snapshot.json
        let config = read_genesis_yaml(&genesis_yaml)
            .map_err(|e| crate::error::Error::Block0Error(Box::new(e)))?;

        let initials: Vec<Initial> = config
            .initial
            .iter()
            .filter(|x| matches!(x, Initial::Fund { .. }))
            .cloned()
            .collect();

        let snapshot = Snapshot { initial: initials };
        let snapshot_ser = serde_json::to_string_pretty(&snapshot)?;

        let mut file = std::fs::File::create(deployment_tree.root_path().join("snapshot.json"))?;
        file.write_all(snapshot_ser.as_bytes())?;
        std::fs::remove_file(genesis_yaml)?;

        println!("Snapshot dumped into {:?}", file);

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct Snapshot {
    pub initial: Vec<Initial>,
}
