use catalyst_toolbox::logs::sentry::{LazySentryLogs, RawLog, SentryLogClient};
use color_eyre::Report;
use jcli_lib::utils::io::open_file_write;

use std::path::PathBuf;

use clap::Parser;
use std::str::FromStr;
use time::OffsetDateTime;
use url::Url;

const DATE_TIME_TAG: &str = "dateCreated";

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Full,
    Latest,
}

#[derive(Debug, Parser)]
pub struct DateFilter {
    // Optional [From, To] date range, start
    #[clap(long, value_parser = parse_datetime_from_rfc3339)]
    from: Option<OffsetDateTime>,
    // Optional [From, To] date range, end
    #[clap(long, value_parser = parse_datetime_from_rfc3339)]
    to: Option<OffsetDateTime>,
}

#[derive(Debug, Parser)]
#[clap(rename_all = "kebab-case")]
pub struct Download {
    /// Sentry logs url
    #[clap(long)]
    url: Url,

    /// Sentry access token
    #[clap(long)]
    token: String,

    /// Path to the ouput file, will be overwritten if exists.
    #[clap(long)]
    out: PathBuf,

    /// Choose between latest logs or full logs download
    #[clap(long, default_value = "latest")]
    mode: Mode,

    #[clap(flatten)]
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

impl Download {
    pub fn exec(self) -> Result<(), Report> {
        let Self {
            url,
            token,
            out,
            mode,
            dates,
        } = self;
        let dates = flip_dates_if_wrong(dates);

        // notice that we use chunk_size of 100. Right now sentry api limits this to 100 per pagination entry
        // so we must request them 100 at a time...
        request_sentry_logs_and_dump_to_file(url, token, mode, dates, out, 100)
    }
}

fn filter_logs_by_date(
    logs: impl Iterator<Item = RawLog>,
    dates: DateFilter,
) -> impl Iterator<Item = RawLog> {
    let DateFilter { from, to } = dates;

    logs.take_while(move |l| {
        let date_time = date_time_from_raw_log(l);
        match from {
            None => true,
            Some(from_date) => date_time >= from_date,
        }
    })
    .filter(move |l| {
        let date_time = date_time_from_raw_log(l);
        match to {
            None => true,
            Some(to_date) => date_time <= to_date,
        }
    })
}

fn flip_dates_if_wrong(dates: DateFilter) -> DateFilter {
    match dates {
        DateFilter {
            from: Some(from),
            to: Some(to),
        } => {
            if from < to {
                DateFilter {
                    from: Some(from),
                    to: Some(to),
                }
            } else {
                DateFilter {
                    from: Some(to),
                    to: Some(from),
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
    chunk_size: usize,
) -> Result<(), Report> {
    let client = SentryLogClient::new(url, token);

    println!("Starting downloading...");
    let logs: Vec<RawLog> = match mode {
        Mode::Full => {
            let sentry_logs = LazySentryLogs::new(client, chunk_size);
            filter_logs_by_date(sentry_logs.into_iter(), dates)
        }
        Mode::Latest => {
            let iter_vec: Box<dyn Iterator<Item = RawLog>> =
                Box::new(client.get_json_logs()?.into_iter());
            filter_logs_by_date(iter_vec, dates)
        }
    }
    .collect();

    dump_logs_to_json(&logs, out.clone())?;
    println!("Finished downloading");
    println!(
        "Downloaded {} log entries at: {}",
        logs.len(),
        out.to_string_lossy()
    );
    Ok(())
}

fn dump_logs_to_json(logs: &[RawLog], out: PathBuf) -> Result<(), Report> {
    let file = open_file_write(&Some(out))?;
    serde_json::to_writer_pretty(file, logs)?;
    Ok(())
}

fn date_time_from_raw_log(l: &RawLog) -> OffsetDateTime {
    parse_datetime_from_rfc3339(
        l.get(DATE_TIME_TAG)
            .expect("A dateCreated entry should be present in sentry logs")
            .as_str()
            .expect("dateCreated should be a str"),
    )
    .expect("A rfc3339 compatible DateTime str")
}

fn parse_datetime_from_rfc3339(datetime: &str) -> Result<OffsetDateTime, time::error::Parse> {
    OffsetDateTime::parse(datetime, &time::format_description::well_known::Rfc3339)
}
