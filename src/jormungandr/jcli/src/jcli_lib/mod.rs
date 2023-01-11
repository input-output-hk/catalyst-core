pub mod address;
pub mod block;
pub mod certificate;
pub mod debug;
pub mod key;
pub mod rest;
pub mod transaction;
pub mod vote;

pub mod utils;

use std::error::Error;
use clap::Parser;

/// Jormungandr CLI toolkit
#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct JCli {
    /// display full version details (software version, source version, targets and compiler used)
    #[clap(long = "full-version")]
    full_version: bool,

    /// display the sources version, allowing to check the source's hash used to compile this executable.
    /// this option is useful for scripting retrieving the logs of the version of this application.
    #[clap(long = "source-version")]
    source_version: bool,

    #[clap(subcommand)]
    command: Option<JCliCommand>,
}

#[allow(clippy::large_enum_variant)]
/// Jormungandr CLI toolkit
#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum JCliCommand {
    /// Key Generation
    #[clap(subcommand)]
    Key(key::Key),
    /// Address tooling and helper
    #[clap(subcommand)]
    Address(address::Address),
    /// Block tooling and helper
    #[clap(subcommand)]
    Genesis(block::Genesis),
    /// Send request to node REST API
    #[clap(subcommand)]
    Rest(rest::Rest),
    /// Build and view offline transaction
    #[clap(subcommand)]
    Transaction(transaction::Transaction),
    /// Debug tools for developers
    #[clap(subcommand)]
    Debug(debug::Debug),
    /// Certificate generation tool
    #[clap(subcommand)]
    Certificate(certificate::Certificate),
    /// Utilities that perform specialized tasks
    #[clap(subcommand)]
    Utils(utils::Utils),
    /// Vote related operations
    #[clap(subcommand)]
    Votes(vote::Vote),
}

impl JCli {
    pub fn exec(self) -> Result<(), Box<dyn Error>> {
        use std::io::Write as _;
        if self.full_version {
            Ok(writeln!(std::io::stdout(), "{}", env!("FULL_VERSION"))?)
        } else if self.source_version {
            Ok(writeln!(std::io::stdout(), "{}", env!("SOURCE_VERSION"))?)
        } else if let Some(cmd) = self.command {
            cmd.exec()
        } else {
            writeln!(std::io::stderr(), "No command, try `--help'")?;
            std::process::exit(1);
        }
    }
}

impl JCliCommand {
    pub fn exec(self) -> Result<(), Box<dyn Error>> {
        use self::JCliCommand::*;
        match self {
            Key(key) => key.exec()?,
            Address(address) => address.exec()?,
            Genesis(genesis) => genesis.exec()?,
            Rest(rest) => rest.exec()?,
            Transaction(transaction) => transaction.exec()?,
            Debug(debug) => debug.exec()?,
            Certificate(certificate) => certificate.exec()?,
            Utils(utils) => utils.exec()?,
            Votes(vote) => vote.exec()?,
        };
        Ok(())
    }
}
