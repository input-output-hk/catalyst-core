use reqwest::Url;
use structopt::StructOpt;

pub const DEFAULT_PUSHWOOSH_API_URL: &str = "https://cp.pushwoosh.com/json/1.3/";

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct ApiParams {
    #[structopt(long, default_value = DEFAULT_PUSHWOOSH_API_URL)]
    pub api_url: Url,
    #[structopt(long)]
    pub access_token: String,
}
