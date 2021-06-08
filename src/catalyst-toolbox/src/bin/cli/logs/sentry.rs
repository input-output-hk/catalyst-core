use super::Error;
use catalyst_toolbox::logs::sentry::SentryLogClient;
use jcli_lib::utils::io::open_file_write;

use std::path::PathBuf;

use structopt::StructOpt;
use url::Url;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum SentryLogs {
    /// Download logs from sentry
    Download(Download),
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
        let Self { url, token, out } = self;
        request_sentry_logs_and_dump_to_file(url, token, out)
    }
}

fn request_sentry_logs_and_dump_to_file(
    url: Url,
    token: String,
    out: PathBuf,
) -> Result<(), Error> {
    let client = SentryLogClient::new(url, token);
    let logs = client.get_json_logs()?;
    let file = open_file_write(&Some(out))?;
    serde_json::to_writer_pretty(file, &logs)?;
    Ok(())
}
