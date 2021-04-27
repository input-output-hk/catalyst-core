use reqwest::Url;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct ApiParams {
    #[structopt(default_value = "https://cp.pushwoosh.com/json/1.3")]
    api_url: Url,
    access_token: String,
}
