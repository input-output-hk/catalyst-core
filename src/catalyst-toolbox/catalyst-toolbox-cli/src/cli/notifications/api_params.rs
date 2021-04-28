use reqwest::Url;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct ApiParams {
    #[structopt(long, default_value = "https://cp.pushwoosh.com/json/1.3")]
    pub api_url: Url,
    #[structopt(long)]
    pub access_token: String,
}
