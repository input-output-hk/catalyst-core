use clap::Parser;
use reqwest::Url;

pub const DEFAULT_PUSHWOOSH_API_URL: &str = "https://cp.pushwoosh.com/json/1.3/";

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct ApiParams {
    #[clap(long, default_value = DEFAULT_PUSHWOOSH_API_URL)]
    pub api_url: Url,
    #[clap(long)]
    pub access_token: String,
}
