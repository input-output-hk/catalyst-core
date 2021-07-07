use catalyst_toolbox::ideascale::{fetch_all, Error, Rewards};

use structopt::StructOpt;

use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab")]
pub struct Import {
    /// Fund number id
    #[structopt(long)]
    fund: usize,

    /// ideascale API token
    #[structopt(long)]
    api_token: String,

    /// Path to a json file containing the related rewards per challenge.
    /// `{ challenge_id:  reward_amount}`
    #[structopt(long)]
    rewards: PathBuf,
}

impl Import {
    fn exec(&self) -> Result<(), Error> {
        let Import {
            fund,
            rewards,
            api_token,
        } = self;

        let runtime = tokio::runtime::Builder::new_multi_thread().build()?;

        let idescale_data =
            futures::executor::block_on(runtime.spawn(fetch_all(*fund, api_token.clone())))??;

        let rewards: Rewards =
            serde_json::from_reader(jcli_lib::utils::io::open_file_read(&Some(rewards))?)?;

        Ok(())
    }
}
