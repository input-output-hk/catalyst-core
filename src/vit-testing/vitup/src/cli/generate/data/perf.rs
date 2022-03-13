use crate::builders::utils::DeploymentTree;
use crate::builders::utils::SessionSettingsExtension;
use crate::builders::VitBackendSettingsBuilder;
use crate::config::read_config;
use crate::mode::standard::generate_database;
use crate::Result;
use glob::glob;
use hersir::config::SessionSettings;
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
        default_value = "../../catalyst-resources/ideascale/fund5/proposals.json"
    )]
    pub proposals: PathBuf,

    /// challenges import json
    #[structopt(
        long = "challenges",
        default_value = "../../catalyst-resources/ideascale/fund5/challenges.json"
    )]
    pub challenges: PathBuf,

    /// funds import json
    #[structopt(
        long = "funds",
        default_value = "../../catalyst-resources/ideascale/fund5/funds.json"
    )]
    pub funds: PathBuf,

    /// reviews import json
    #[structopt(
        long = "reviews",
        default_value = "../../catalyst-resources/ideascale/fund5/reviews.json"
    )]
    pub reviews: PathBuf,

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

        let session_settings = SessionSettings::from_dir(&self.output_directory);

        let mut config = read_config(&self.config)?;

        if let Some(ref snapshot) = self.snapshot {
            config.extend_from_initials_file(snapshot)?;
        }

        if !self.output_directory.exists() {
            std::fs::create_dir_all(&self.output_directory)?;
        }

        let deployment_tree = DeploymentTree::new(&self.output_directory);

        let (controller, vit_parameters) = VitBackendSettingsBuilder::default()
            .skip_qr_generation()
            .config(&config)
            .session_settings(session_settings)
            .build()?;

        let template_generator = ExternalValidVotingTemplateGenerator::new(
            self.proposals.clone(),
            self.challenges.clone(),
            self.funds.clone(),
            self.reviews.clone(),
        )
        .unwrap();

        generate_database(&deployment_tree, vit_parameters, template_generator);

        self.move_single_user_secrets(
            &deployment_tree,
            deployment_tree.root_path().join("single"),
        )?;
        self.split_secrets(&deployment_tree)?;

        println!(
            "voteplan ids: {:?}",
            controller
                .defined_vote_plans()
                .iter()
                .map(|x| x.id())
                .collect::<Vec<String>>()
        );

        config.print_report();
        Ok(())
    }

    fn move_single_user_secrets<P: AsRef<Path>>(
        &self,
        tree: &DeploymentTree,
        output_folder: P,
    ) -> Result<()> {
        let pattern = tree.wallet_search_pattern();
        std::fs::create_dir_all(&output_folder).unwrap();
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

    fn split_secrets(&self, tree: &DeploymentTree) -> Result<()> {
        let pattern = tree.wallet_search_pattern();
        let secrets: Vec<PathBuf> = (0..self.parts)
            .into_iter()
            .map(|id| {
                let folder = tree
                    .root_path()
                    .join("secrets".to_owned() + &id.to_string());
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
