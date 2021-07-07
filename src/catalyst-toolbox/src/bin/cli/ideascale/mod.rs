use catalyst_toolbox::ideascale::{
    build_challenges, build_fund, build_proposals, fetch_all, Error, Rewards,
};

use structopt::StructOpt;

use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, StructOpt)]
pub enum Idescale {
    Import(Import),
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab")]
pub struct Import {
    /// Fund number id
    #[structopt(long)]
    fund: usize,

    /// ideascale API token
    #[structopt(long)]
    api_token: String,

    threshold: i64,
    rewards_info: String,
    chain_vote_type: String,

    /// Path to a json file containing the related rewards per challenge.
    /// `{ challenge_id:  reward_amount}`
    #[structopt(long)]
    rewards: PathBuf,

    #[structopt(long)]
    save_folder: PathBuf,
}

impl Idescale {
    pub fn exec(&self) -> Result<(), Error> {
        match self {
            Idescale::Import(import) => import.exec(),
        }
    }
}

impl Import {
    fn exec(&self) -> Result<(), Error> {
        let Import {
            fund,
            rewards,
            api_token,
            threshold,
            rewards_info,
            chain_vote_type,
            save_folder,
        } = self;

        let runtime = tokio::runtime::Builder::new_multi_thread().build()?;

        let idescale_data =
            futures::executor::block_on(runtime.spawn(fetch_all(*fund, api_token.clone())))??;

        let rewards: Rewards =
            serde_json::from_reader(jcli_lib::utils::io::open_file_read(&Some(rewards))?)?;

        let funds = build_fund(&idescale_data, *threshold, rewards_info.clone());
        let challenges = build_challenges(&idescale_data, &rewards);
        let proposals = build_proposals(&idescale_data, chain_vote_type, *fund);

        dump_content_to_file(
            funds,
            save_folder
                .join(format!("fund{}_funds.json", fund))
                .as_path(),
        )?;

        dump_content_to_file(
            challenges,
            save_folder
                .join(format!("fund{}_challenges.json", fund))
                .as_path(),
        )?;

        dump_content_to_file(
            proposals,
            save_folder
                .join(format!("fund{}_proposals.json", fund))
                .as_path(),
        )?;

        Ok(())
    }
}

fn dump_content_to_file(content: impl Serialize, file_path: &Path) -> Result<(), Error> {
    let writer = jcli_lib::utils::io::open_file_write(&Some(file_path))?;
    serde_json::to_writer_pretty(writer, &content).map_err(Error::SerdeError)
}
