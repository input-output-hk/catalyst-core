use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use std::fmt;
use std::thread;
use std::time::Instant;
use clap::Parser;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use thiserror::Error;
use valgrind::{Error as ValgrindError, ValgrindClient};
use vit_servicing_station_lib::utils::datetime::unix_timestamp_to_datetime;

#[derive(Parser, Debug)]
#[clap(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct DeploymentValidateCommand {
    /// target address
    #[clap(long = "address")]
    pub address: String,
}

#[derive(Debug, EnumIter, Copy, Clone)]
enum Check {
    Fund,
    Proposals,
    Settings,
    VotePlan,
    Reviews,
    Challenges,
    BadGateway,
    Times,
}

impl fmt::Display for Check {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Check::Fund => write!(f, "[vit-ss] Fund endpoint check"),
            Check::Proposals => write!(f, "[vit-ss] Proposals endpoint check"),
            Check::Settings => write!(f, "[jormungandr] Settings endpoint check"),
            Check::VotePlan => write!(f, "[jormungandr] Vote plan endpoint"),
            Check::Reviews => write!(f, "[vit-ss] Reviews endpoint"),
            Check::Challenges => write!(f, "[vit-ss] Challenges endpoint"),
            Check::BadGateway => write!(f, "[proxy] Bad gateway check"),
            Check::Times => write!(f, "[vit-ss] Fund dates check"),
        }
    }
}

impl Check {
    pub fn execute(
        &self,
        wallet_backend: &ValgrindClient,
    ) -> std::result::Result<std::time::Duration, CheckError> {
        let mut started = Instant::now();

        match self {
            Self::Fund => wallet_backend
                .funds()
                .map(|_| started.elapsed())
                .map_err(Into::into),
            Self::Proposals => wallet_backend
                .proposals("direct")
                .map(|_| started.elapsed())
                .map_err(Into::into),
            Self::Settings => wallet_backend
                .settings()
                .map(|_| started.elapsed())
                .map_err(Into::into),
            Self::VotePlan => wallet_backend
                .vote_plan_statuses()
                .map(|_| started.elapsed())
                .map_err(Into::into),
            Self::Reviews => {
                let proposals = wallet_backend.proposals("direct")?;
                started = Instant::now();
                wallet_backend
                    .review(&proposals[0].proposal.proposal_id)
                    .map(|_| started.elapsed())
                    .map_err(Into::into)
            }
            Self::Challenges => wallet_backend
                .challenges()
                .map(|_| started.elapsed())
                .map_err(Into::into),
            Self::BadGateway => {
                let left = wallet_backend.funds()?;
                let right = wallet_backend.funds()?;

                if left != right {
                    Err(CheckError::Assert(
                        "two calls to funds return different response".to_string(),
                    ))
                } else {
                    Ok(started.elapsed())
                }
            }
            Self::Times => {
                let fund = wallet_backend.funds()?;

                let registration_date = unix_timestamp_to_datetime(fund.registration_snapshot_time);
                let fund_start_date = unix_timestamp_to_datetime(fund.fund_start_time);
                let fund_end_date = unix_timestamp_to_datetime(fund.fund_end_time);
                let next_fund_date = unix_timestamp_to_datetime(fund.next_fund_start_time);

                if registration_date > fund_start_date {
                    return Err(CheckError::Assert(
                        "registration_date is further in the future than fund_start_date"
                            .to_string(),
                    ));
                }
                if fund_start_date > fund_end_date {
                    return Err(CheckError::Assert(
                        "fund_start_date is further in the future than fund_end_date".to_string(),
                    ));
                }

                if fund_end_date > next_fund_date {
                    return Err(CheckError::Assert(
                        "fund_end_date is further in the future than next_fund_date".to_string(),
                    ));
                }
                Ok(started.elapsed())
            }
        }
    }
}

impl DeploymentValidateCommand {
    pub fn exec(self) -> Result<(), CheckError> {
        let started = Instant::now();
        let spinner_style = ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{prefix:.bold.dim} {spinner} {wide_msg}");

        let commands: Vec<Check> = Check::iter().collect();
        let len = commands.len();
        let m = MultiProgress::new();

        let _handles: Vec<_> = commands
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, command)| {
                let pb = m.add(ProgressBar::new(1));
                pb.set_style(spinner_style.clone());
                pb.set_prefix(&format!("[{}/{}]", i + 1, len));
                pb.set_message(&format!("{}...In progress", command));

                let address = self.address.clone();
                let wallet_backend = ValgrindClient::new(address, Default::default()).unwrap();
                thread::spawn(move || {
                    let finish_style =
                        ProgressStyle::default_spinner().template("{prefix:.bold.dim} {wide_msg}");

                    let result = command.execute(&wallet_backend);

                    pb.set_style(finish_style);

                    match result {
                        Ok(elapsed) => pb.finish_with_message(&format!(
                            "[Passed] {}. Duration: {} ms",
                            command,
                            elapsed.as_millis()
                        )),
                        Err(error) => pb.finish_with_message(&format!(
                            "[Failed] {}. Error: {}",
                            command, error
                        )),
                    };
                })
            })
            .collect();

        m.join().unwrap();

        println!();
        println!("Done in {}", HumanDuration(started.elapsed()));

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum CheckError {
    #[error(transparent)]
    WalletBackend(#[from] ValgrindError),
    #[error("{0}")]
    Assert(String),
}
