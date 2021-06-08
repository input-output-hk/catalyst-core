use super::Error;
use catalyst_toolbox::logs::sentry::{LazySentryLogs, RawLog, SentryLogClient};
use jcli_lib::utils::io::open_file_write;

use std::path::PathBuf;

use chrono::{DateTime, FixedOffset};
use std::str::FromStr;
use structopt::StructOpt;
use url::Url;

const DATE_TIME_TAG: &str = "dateCreated";

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
pub struct DateFilter {
    #[structopt(parse(try_from_str = DateTime::parse_from_rfc3339))]
    init: Option<DateTime<FixedOffset>>,
    #[structopt(parse( try_from_str = DateTime::parse_from_rfc3339))]
    end: Option<DateTime<FixedOffset>>,
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Download {
    /// Sentry logs url
    #[structopt(long)]
    url: Url,

    /// Sentry access token
    #[structopt(long)]
    token: String,

    /// Path to the ouput file, will be overwritten if exists.
    #[structopt(long)]
    out: PathBuf,

    /// Choose between latest logs or full logs download
    #[structopt(long, default_value = "latest")]
    mode: Mode,

    #[structopt(flatten)]
    dates: DateFilter,
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
            dates,
        } = self;
        let dates = flip_dates_if_wrong(dates);

        request_sentry_logs_and_dump_to_file(url, token, mode, dates, out)
    }
}

fn filter_logs_by_date(
    logs: impl Iterator<Item = RawLog>,
    dates: DateFilter,
) -> impl Iterator<Item = RawLog> {
    let DateFilter { init, end } = dates;

    logs.filter(move |l| {
        let date_time = DateTime::parse_from_rfc3339(
            l.get(DATE_TIME_TAG)
                .expect("A dateCreated entry should be present in sentry logs")
                .as_str()
                .expect("dateCreated should be a str"),
        )
        .expect("A rfc3339 compatible DateTime str");
        match init {
            None => true,
            Some(init_date) => init_date > date_time,
        }
    })
    .take_while(move |l| {
        let date_time = DateTime::parse_from_rfc3339(
            l.get(DATE_TIME_TAG)
                .expect("A dateCreated entry should be present in sentry logs")
                .as_str()
                .expect("dateCreated should be a str"),
        )
        .expect("A rfc3339 compatible DateTime str");
        match end {
            None => true,
            Some(end_date) => end_date < date_time,
        }
    })
}

fn flip_dates_if_wrong(dates: DateFilter) -> DateFilter {
    match dates {
        DateFilter {
            init: Some(init),
            end: Some(end),
        } => {
            if init < end {
                DateFilter {
                    init: Some(init),
                    end: Some(end),
                }
            } else {
                DateFilter {
                    init: Some(end),
                    end: Some(init),
                }
            }
        }
        other => other,
    }
}

fn request_sentry_logs_and_dump_to_file(
    url: Url,
    token: String,
    mode: Mode,
    dates: DateFilter,
    out: PathBuf,
) -> Result<(), Error> {
    let client = SentryLogClient::new(url, token);
    let logs: Vec<RawLog> = match mode {
        Mode::Full => {
            let sentry_logs = LazySentryLogs::new(client, 1000);
            filter_logs_by_date(sentry_logs.into_iter(), dates).collect()
        }
        Mode::Latest => filter_logs_by_date(client.get_json_logs()?.into_iter(), dates).collect(),
    };

    dump_logs_to_json(&logs, out)
}

fn dump_logs_to_json(logs: &[RawLog], out: PathBuf) -> Result<(), Error> {
    let file = open_file_write(&Some(out))?;
    serde_json::to_writer_pretty(file, logs)?;
    Ok(())
}
