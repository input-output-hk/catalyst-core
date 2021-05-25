mod kedqr;
mod logs;
mod notifications;
mod recovery;
mod rewards;

use std::error::Error;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Cli {
    /// display full version details (software version, source version, targets and compiler used)
    #[structopt(long = "full-version")]
    full_version: bool,

    /// display the sources version, allowing to check the source's hash used to compile this executable.
    /// this option is useful for scripting retrieving the logs of the version of this application.
    #[structopt(long = "source-version")]
    source_version: bool,

    #[structopt(subcommand)]
    command: Option<CatalystCommand>,
}

#[allow(clippy::large_enum_variant)]
#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum CatalystCommand {
    /// Rewards related operations
    Rewards(rewards::Rewards),
    /// Send push notification to pushwoosh service
    PushNotification(notifications::PushNotifications),
    /// Tally recovery utility
    Recover(recovery::Recover),
    /// Download, compare and get stats from sentry and persistent fragment logs
    Logs(logs::Logs),
    /// Generate qr codes
    Kedqr(kedqr::QRcodeApp),
}

impl Cli {
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

impl CatalystCommand {
    pub fn exec(self) -> Result<(), Box<dyn Error>> {
        use self::CatalystCommand::*;
        match self {
            Rewards(rewards) => rewards.exec()?,
            PushNotification(notifications) => notifications.exec()?,
            Recover(recover) => recover.exec()?,
            Logs(logs) => logs.exec()?,
            Kedqr(kedqr) => kedqr.exec()?,
        };
        Ok(())
    }
}
