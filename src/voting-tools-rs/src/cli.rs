use crate::{
    data::{DbHost, DbName, DbPass, DbUser, NetworkId, SlotNo, VotingPurpose},
    error::InvalidRegistration,
};
use chrono::Utc;
use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

#[derive(Debug, Parser)]
#[cfg_attr(test, derive(PartialEq))]
#[non_exhaustive]
#[clap(about = "Create a voting power snapshot")]
/// CLI arguments for snapshot tool
pub struct Args {
    /// Name of the cardano-db-sync database
    #[clap(long, default_value = "cexplorer")]
    pub db: DbName,

    /// User to connect to the  cardano-db-sync database with
    #[clap(long, default_value = "cexplorer")]
    pub db_user: DbUser,

    /// Host for the cardano-db-sync database connection
    #[clap(long, default_value = "/run/postgresql")]
    pub db_host: DbHost,

    /// Password for the cardano-db-sync database connection
    #[clap(long)]
    pub db_pass: Option<DbPass>,

    /// Lower bound for slot number to be included in queries
    #[clap(long)]
    pub min_slot: Option<SlotNo>,

    /// Upper bound for slot number to be included in queries
    #[clap(long)]
    pub max_slot: Option<SlotNo>,

    /// File to output the signed transaction to
    #[clap(long, short = 'o')]
    pub out_file: PathBuf,

    /// This parameter should be used only for voting tool dry runs or internal testing
    #[clap(subcommand)]
    pub dry_run: Option<DryRunCommand>,

    /// The network to validate signatures against
    #[clap(long, default_value = NetworkId::Mainnet)]
    pub network_id: NetworkId,

    /// The voting purpose to use in queries
    #[clap(long, default_value = VotingPurpose::CATALYST)]
    pub expected_voting_purpose: VotingPurpose,

    /// Enable Multiple delegations in CIP-36 registrations
    #[clap(long)]
    pub enable_cip36_multiple_delegations: bool,
}

/// Sub command for internal testing or dry runs
#[derive(Subcommand, Debug, PartialEq)]
pub enum DryRunCommand {
    /// Sub command for internal testing or dry runs
    DryRun {
        #[clap(long)]
        /// Mock json file containing content of db sync db. This parameter should be used only for
        /// voting tool dry runs
        mock_json_file: PathBuf,
    },
}

/// If there are errors, we want to notify the user, but it's not really actionable, so we provide
/// the option to silence the error via env var
///
/// # Errors
///
/// Errors if there are any IO errors writing logs
pub fn show_error_warning(errors: &[InvalidRegistration]) -> Result<()> {
    let num_errs = errors.len();

    if num_errs == 0 || std::env::var("VOTING_TOOL_SUPPRESS_WARNINGS").unwrap() == "1" {
        return Ok(());
    }

    warn!("{num_errs} rows generated errors, set `VOTING_TOOL_SUPPRESS_WARNINGS=1 to suppress this warning");

    let path = error_log_file()?;
    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);

    for e in errors {
        writeln!(&mut writer, "{e:?}")?;
    }

    warn!("error logs have been written to {}", path.to_string_lossy());

    Ok(())
}

fn error_log_file() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().expect("no home dir found to write logs");
    let error_dir = home_dir.join(".voting_tool_logs");
    std::fs::create_dir_all(&error_dir)?;

    let now = Utc::now();
    let log_file = error_dir.join(now.format("%Y-%m-%d--%H-%M-%S").to_string());

    Ok(log_file)
}

#[cfg(test)]
mod tests {
    use microtype::SecretMicrotype;

    use super::*;

    #[test]
    fn can_parse_all_values() {
        let args = Args::parse_from([
            "binary_name",
            "--db",
            "db_name",
            "--db-user",
            "db_user",
            "--db-host",
            "localhost",
            "--db-pass",
            "super secret password",
            "--min-slot",
            "123",
            "--max-slot",
            "234",
            "-o",
            "some/path",
            "--expected-voting-purpose",
            "0",
            "--network-id",
            "mainnet",
            "--enable-cip36-multiple-delegations",
        ]);

        assert_eq!(
            args,
            Args {
                db: "db_name".into(),
                db_user: "db_user".into(),
                db_host: "localhost".into(),
                db_pass: Some(DbPass::new("super secret password".to_string())),
                min_slot: Some(123.into()),
                max_slot: Some(234.into()),
                out_file: "some/path".into(),
                dry_run: None,
                network_id: NetworkId::Mainnet,
                expected_voting_purpose: VotingPurpose::CATALYST,
                enable_cip36_multiple_delegations: true,
            }
        );
    }

    #[test]
    fn can_parse_only_required_values() {
        let args = Args::parse_from(["binary_name", "-o", "some/path"]);

        assert_eq!(args.out_file, PathBuf::from("some/path"));
    }
}
