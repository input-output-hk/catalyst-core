use crate::builders::utils::DeploymentTree;
use crate::Result;
use assert_fs::TempDir;
use clap::Parser;
use diffy::create_patch;
use diffy::PatchFormatter;
use jormungandr_automation::testing::block0::decode_block0;
use jortestkit::prelude::read_file;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use valgrind::ValgrindClient;
use vit_servicing_station_tests::common::startup::db::DbBuilder;
use vit_servicing_station_tests::common::startup::server::ServerBootstrapper;

#[derive(Parser, Debug)]
pub struct DiffCommand {
    /// Local environment to compare
    #[clap(short = 'l', long = "local", default_value = "./data/")]
    pub local: PathBuf,

    /// Vit servicing station server binary
    #[clap(long = "vit-station", default_value = "vit-servicing-station-server")]
    pub vit_station: PathBuf,

    /// Target environment to compare
    #[clap(short = 't', long = "target")]
    pub target: String,

    /// Output file. If not defined it will output to stdout
    #[clap(short = 'o', long = "output")]
    pub output: Option<PathBuf>,

    /// Apply coloring during diff
    #[clap(short = 'c', long = "color")]
    pub color: bool,
}

impl DiffCommand {
    pub fn exec(self) -> Result<()> {
        std::env::set_var("RUST_BACKTRACE", "full");
        let remote = TempDir::new().unwrap();
        let deployment_tree = DeploymentTree::new(&self.local);
        let local_genesis_yaml = deployment_tree.genesis_path();
        let remote_client = ValgrindClient::new(self.target.clone(), Default::default())?;

        let remote_genesis_yaml = remote.path().join("genesis_remote.yaml");

        decode_block0(remote_client.block0()?, remote_genesis_yaml.clone()).map_err(|e| crate::error::Error::Block0Error(Box::new(e)))?;

        let local_genesis_content = read_file(&local_genesis_yaml)?;
        let remote_genesis_content = read_file(remote_genesis_yaml)?;

        let db_url = DbBuilder::new().build().unwrap();

        let server = ServerBootstrapper::new()
            .with_db_path(db_url)
            .start_with_exe(&remote, self.vit_station.clone())?;

        let local_funds = server.rest_client().funds()?;
        let remote_funds = remote_client.funds()?;

        let local_funds_content = serde_json::to_string_pretty(&local_funds)?;
        let remote_funds_content = serde_json::to_string_pretty(&remote_funds)?;

        let group = "group";
        let mut local_proposals = server.rest_client().proposals(group)?;
        let mut remote_proposals = remote_client.proposals(group)?;

        let local_proposals_content = serde_json::to_string_pretty(
            &local_proposals.sort_by(|a, b| b.proposal.internal_id.cmp(&a.proposal.internal_id)),
        )?;
        let remote_proposals_content = serde_json::to_string_pretty(
            &remote_proposals.sort_by(|a, b| b.proposal.internal_id.cmp(&a.proposal.internal_id)),
        )?;

        let address = server.settings().address;

        let patch = create_patch(&remote_genesis_content, &local_genesis_content);
        let f = get_formatter(self.color);
        let block0_diff_header = format!(
            "diff http://{}/api/v0/block0 {:?}\n",
            self.target, local_genesis_yaml
        );
        let block0_diff = format!("{}\n", f.fmt_patch(&patch));

        let patch = create_patch(&local_funds_content, &remote_funds_content);
        let f = get_formatter(self.color);
        let fund_diff_header = format!(
            "diff http://{}/api/v0/fund {}/api/v0/fund\n",
            self.target, address
        );
        let fund_diff = format!("{}\n", f.fmt_patch(&patch));

        let patch = create_patch(&local_proposals_content, &remote_proposals_content);
        let f = get_formatter(self.color);
        let proposals_diff_header = format!(
            "diff http://{}/api/v0/block0 {}/api/v0/proposals\n",
            self.target, address
        );
        let proposals_diff = format!("{}\n", f.fmt_patch(&patch));

        if let Some(output) = self.output {
            let mut file = File::create(output)?;
            file.write_all(block0_diff_header.as_bytes())?;
            file.write_all(block0_diff.as_bytes())?;
            file.write_all(fund_diff_header.as_bytes())?;
            file.write_all(fund_diff.as_bytes())?;
            file.write_all(proposals_diff_header.as_bytes())?;
            file.write_all(proposals_diff.as_bytes())?;
        } else {
            print!("{}", block0_diff_header);
            print!("{}", block0_diff);
            print!("{}", fund_diff_header);
            print!("{}", fund_diff);
            print!("{}", proposals_diff_header);
            print!("{}", proposals_diff);
        }
        Ok(())
    }
}

fn get_formatter(color: bool) -> PatchFormatter {
    let mut f = PatchFormatter::new();
    if color {
        f = f.with_color();
    }
    f
}
