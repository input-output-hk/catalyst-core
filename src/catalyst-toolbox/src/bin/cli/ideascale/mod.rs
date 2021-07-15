use catalyst_toolbox::ideascale::{
    build_challenges, build_fund, build_proposals, fetch_all, Error,
};

use structopt::StructOpt;

use serde::Serialize;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug)]
pub enum VoteType {
    Public,
    Private,
}

#[derive(Debug, StructOpt)]
pub enum Ideascale {
    Import(Import),
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab")]
pub struct Import {
    /// Fund number id
    #[structopt(long)]
    fund: usize,

    /// Fund goal explanation
    #[structopt(long)]
    fund_goal: String,

    /// Stage label: stage identifiers that links to assessments scores in ideascale
    #[structopt(long, default_value = "Assess")]
    stage_label: String,

    /// ideascale API token
    #[structopt(long, env = "IDEASCALE_API_TOKEN")]
    api_token: String,

    /// Fund approval threshold setting
    #[structopt(long)]
    threshold: i64,

    /// either "public" or "private"
    #[structopt(long)]
    chain_vote_type: VoteType,

    /// Path to folder where fund, challenges and proposals json files will be dumped
    #[structopt(long)]
    output_dir: PathBuf,
}

impl Ideascale {
    pub fn exec(&self) -> Result<(), Error> {
        match self {
            Ideascale::Import(import) => import.exec(),
        }
    }
}

impl Import {
    fn exec(&self) -> Result<(), Error> {
        let Import {
            fund,
            fund_goal,
            stage_label,
            api_token,
            threshold,
            chain_vote_type,
            output_dir: save_folder,
        } = self;

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .enable_time()
            .build()?;

        let idescale_data = runtime.block_on(fetch_all(*fund, &stage_label, api_token.clone()))?;

        let funds = build_fund(*fund as i32, fund_goal.clone(), *threshold);
        let challenges = build_challenges(*fund as i32, &idescale_data);
        let proposals = build_proposals(
            &idescale_data,
            &challenges,
            &chain_vote_type.to_string(),
            *fund,
        );

        let challenges: Vec<_> = challenges.values().collect();

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

impl FromStr for VoteType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "public" => Ok(Self::Public),
            "private" => Ok(Self::Private),
            _ => Err("Only 'public' or 'private' are correct values for VoteType"),
        }
    }
}

impl Display for VoteType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            VoteType::Public => "public",
            VoteType::Private => "private",
        };
        write!(f, "{}", s)
    }
}

fn dump_content_to_file(content: impl Serialize, file_path: &Path) -> Result<(), Error> {
    let writer = jcli_lib::utils::io::open_file_write(&Some(file_path))?;
    serde_json::to_writer_pretty(writer, &content).map_err(Error::SerdeError)
}
