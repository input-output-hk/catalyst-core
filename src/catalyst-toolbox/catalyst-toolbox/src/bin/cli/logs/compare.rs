use catalyst_toolbox::logs::compare::{compare_logs, LogCmpStats};
use catalyst_toolbox::logs::sentry::{RawLog, SentryFragmentLog};
use chain_core::property::Fragment;
use clap::Parser;
use color_eyre::Report;
use jcli_lib::utils::io;
use jormungandr_lib::interfaces::{
    load_persistent_fragments_logs_from_folder_path, PersistentFragmentLog,
};
use serde::de::DeserializeOwned;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(rename_all = "kebab-case")]
pub struct Compare {
    #[clap(long)]
    sentry_logs: PathBuf,

    #[clap(long)]
    permanent_logs: PathBuf,
}

impl Compare {
    pub fn exec(self) -> Result<(), Report> {
        let Self {
            sentry_logs,
            permanent_logs,
        } = self;
        let sentry_logs: Vec<RawLog> = load_logs_from_file(sentry_logs)?;

        let sentry_logs_data: Vec<SentryFragmentLog> = sentry_logs
            .iter()
            .enumerate()
            .filter_map(
                |(i, raw_log)| match raw_log.get("message").and_then(|v| v.as_str()) {
                    None => {
                        // if we could deserialize should be safe to re-serialize it again
                        eprintln!(
                            "couldn't load sentry log for entry {}: {}",
                            i,
                            serde_json::to_string(raw_log).unwrap()
                        );
                        None
                    }
                    Some(value) => match value.parse::<SentryFragmentLog>() {
                        Ok(log) => Some(log),
                        Err(e) => {
                            eprintln!(
                                "couldn't load sentry log for entry {} with message '{}' due to: {:?}",
                                i, value, e
                            );
                            None
                        }
                    },
                },
            )
            .collect();

        let permanent_logs_data: Vec<PersistentFragmentLog> =
            load_persistent_fragments_logs_from_folder_path(&permanent_logs)?
                .enumerate()
                .filter_map(|(i, res)| match res {
                    Ok(log) => Some(log),
                    Err(e) => {
                        eprintln!(
                            "Error deserializing persistent fragment log entry {}: {:?}",
                            i, e
                        );
                        None
                    }
                })
                .collect();

        let cmp_result = compare_logs(&sentry_logs_data, &permanent_logs_data);
        print_results(&cmp_result);
        Ok(())
    }
}

pub fn load_logs_from_file<L: DeserializeOwned>(path: PathBuf) -> Result<Vec<L>, Report> {
    let reader = io::open_file_read(&Some(path))?;
    Ok(serde_json::from_reader(reader)?)
}

pub fn print_results(results: &LogCmpStats) {
    let LogCmpStats {
        sentry_logs_size,
        fragment_logs_size,
        duplicated_sentry_logs,
        duplicated_fragment_logs,
        fragment_ids_differ,
        unhandled_fragment_logs,
    } = results;
    for (unhandled_fragment, e) in unhandled_fragment_logs {
        eprintln!(
            "unable to load fragment information from fragment id {} due to: {:?}",
            unhandled_fragment.id(),
            e
        );
    }
    println!("Sentry logs size {}", sentry_logs_size);
    println!("Fragment logs size {}", fragment_logs_size);
    println!("Duplicated sentry logs {}", duplicated_sentry_logs);
    println!("Duplicated fragments logs {}", duplicated_fragment_logs);
    if !fragment_ids_differ.is_empty() {
        println!("Non matching (sentry over persistent logs) fragment id's:");
        for id in fragment_ids_differ {
            println!("\t{}", id);
        }
    } else {
        println!("All fragment ids match (sentry over persistent logs)");
    }
}
