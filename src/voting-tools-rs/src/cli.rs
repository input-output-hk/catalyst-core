use crate::data::{DbHost, DbName, DbPass, DbUser, SlotNo};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
    pub min_slot_no: Option<SlotNo>,

    /// Upper bound for slot number to be included in queries
    #[clap(long)]
    pub max_slot_no: Option<SlotNo>,

    /// File to output the signed transaction to
    #[clap(long, short = 'o')]
    pub out_file: PathBuf,

    /// Whether to pretty-print the json
    #[clap(long, short = 'p')]
    pub pretty: bool,

    /// This parameter should be used only for voting tool dry runs or internal testing
    #[clap(subcommand)]
    pub dry_run: Option<DryRunCommand>,
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
            "--min-slot-no",
            "123",
            "--max-slot-no",
            "234",
            "-o",
            "some/path",
            "-p",
        ]);

        assert_eq!(
            args,
            Args {
                db: "db_name".into(),
                db_user: "db_user".into(),
                db_host: "localhost".into(),
                db_pass: Some(DbPass::new("super secret password".to_string())),
                min_slot_no: Some(123.into()),
                max_slot_no: Some(234.into()),
                out_file: "some/path".into(),
                dry_run: None,
                pretty: true,
            }
        );
    }

    #[test]
    fn can_parse_only_required_values() {
        let args = Args::parse_from(["binary_name", "-o", "some/path"]);

        assert_eq!(args.out_file, PathBuf::from("some/path"));
        assert_eq!(args.pretty, false);
    }
}
