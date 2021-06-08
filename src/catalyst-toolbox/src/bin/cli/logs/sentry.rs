use super::Error;
use catalyst_toolbox::logs::sentry::{LazySentryLogs, RawLog, SentryLogClient};
use jcli_lib::utils::io::open_file_write;

use std::path::PathBuf;

use std::str::FromStr;
use structopt::StructOpt;
use url::Url;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum SentryLogs {
    /// Download logs from sentry
    Download(Download),
}

pub enum Mode {
    Full,
    Latest,
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Download {
    #[structopt(long)]
    url: Url,
    #[structopt(long)]
    token: String,
    #[structopt(long)]
    out: PathBuf,
    #[structopt(long, default_value = "latest")]
    mode: Mode,
}

impl FromStr for Mode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "f" | "full" => Ok(Self::Full),
            "l" | "latest" => Ok(Self::Latest),
            _ => Err(format!(
                "Could not parse Mode {}. Any of 'f', 'full', 'l' or 'latest' is required",
                s
            )),
        }
    }
}

impl SentryLogs {
    pub fn exec(self) -> Result<(), Error> {
        match self {
            SentryLogs::Download(download) => download.exec(),
        }
    }
}

impl Download {
    pub fn exec(self) -> Result<(), Error> {
        let Self {
            url,
            token,
            out,
            mode,
        } = self;
        request_sentry_logs_and_dump_to_file(url, token, mode, out)
    }
}

fn request_sentry_logs_and_dump_to_file(
    url: Url,
    token: String,
    mode: Mode,
    out: PathBuf,
) -> Result<(), Error> {
    let client = SentryLogClient::new(url, token);
    let logs: Vec<RawLog> = match mode {
        Mode::Full => {
            let sentry_logs = LazySentryLogs::new(client, 1000);
            sentry_logs.into_iter().collect()
        }
        Mode::Latest => client.get_json_logs()?,
    };

    dump_logs_to_json(&logs, out)
}

fn dump_logs_to_json(logs: &[RawLog], out: PathBuf) -> Result<(), Error> {
    let file = open_file_write(&Some(out))?;
    serde_json::to_writer_pretty(file, logs)?;
    Ok(())
}
