use super::CliController;
use bech32::ToBase32;
use catalyst_toolbox::kedqr::decode;
use catalyst_toolbox::kedqr::KeyQrCode;
use catalyst_toolbox::kedqr::KeyQrCodeError;
use chain_impl_mockchain::block::BlockDate;
use clap::Parser;
use iapyx::ControllerBuilderError;
use iapyx::ControllerError;
use jcli_lib::key::read_bech32;
use jormungandr_automation::jormungandr::RestError;
use jormungandr_lib::crypto::hash::Hash;
use jormungandr_lib::interfaces::{AccountVotes, FragmentStatus};
use prettytable::{
    format::{FormatBuilder, LinePosition, LineSeparator},
    row, Table,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;
use thor::cli::{Alias, Connection};
use url::Url;
use valgrind::ProposalExtension;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
use wallet_core::Choice;

///
///
/// Command line wallet for testing Catalyst
///
#[derive(Parser, Debug)]
pub enum IapyxCommand {
    /// Sets node rest API address. Verifies connection on set.
    Connect(Connect),
    /// Gets address of wallet in bech32 format
    Address,
    /// Prints proposals available to vote on
    Proposals(Proposals),
    /// Prints wallet status (balance/spending counters/tokens)
    Status,
    /// Clears pending transactions to confirm. In case if expiration occurred
    ClearTx,
    /// Confirms successful transaction
    ConfirmTx,
    /// Pulls wallet data from the catalyst backend
    Refresh,
    /// Prints entire fragment logs from the node
    Logs,
    /// Prints information about voting funds
    Funds,
    /// Prints pending or already sent fragments statuses
    Statuses,
    /// Sends votes to backend
    #[clap(subcommand)]
    Vote(Vote),
    /// Prints history of votes
    Votes(Votes),
    /// Prints pending transactions (not confirmed)
    PendingTransactions,
    /// Allows to manage wallets: add/remove/select operations
    #[clap(subcommand)]
    Wallets(Wallets),
}

impl IapyxCommand {
    pub fn exec(self, mut model: CliController) -> Result<(), IapyxCommandError> {
        match self {
            IapyxCommand::Wallets(wallets) => wallets.exec(model),
            IapyxCommand::Connect(connect) => connect.exec(model),
            IapyxCommand::Proposals(proposals) => proposals.exec(model),
            IapyxCommand::Address => {
                let wallet = model.wallets().wallet()?;

                let mut table = table();
                table.set_titles(row!["alias", "address", "account id",]);
                table.add_row(row![
                    model
                        .wallets()
                        .default_alias()
                        .unwrap_or(&"undefined".to_string()),
                    wallet.address_readable()?.to_string(),
                    wallet.id()?
                ]);
                table.printstd();
                Ok(())
            }
            IapyxCommand::Status => {
                let account_state = model.account_state()?;
                let mut table = table();
                table.add_row(row![
                    "delegation",
                    format!("{:#?}", account_state.delegation())
                ]);
                table.add_row(row!["value", account_state.value()]);
                table.add_row(row![
                    "spending counters",
                    format!("{:#?}", account_state.counters())
                ]);
                table.add_row(row![
                    "rewards",
                    format!("{:#?}", account_state.last_rewards())
                ]);
                table.add_row(row!["tokens", format!("{:#?}", account_state.tokens())]);
                table.printstd();
                Ok(())
            }
            IapyxCommand::PendingTransactions => {
                let mut table = table();
                table.set_titles(row!["index", "hash"]);
                model
                    .wallets()
                    .wallet()?
                    .pending_tx
                    .iter()
                    .enumerate()
                    .for_each(|(idx, fragment_id)| {
                        table.add_row(row![(idx + 1), fragment_id]);
                    });
                table.printstd();
                Ok(())
            }
            IapyxCommand::Vote(vote) => vote.exec(model),
            IapyxCommand::ConfirmTx => {
                model.confirm_txs()?;
                model.save_config().map_err(Into::into)
            }
            IapyxCommand::ClearTx => {
                model.clear_txs()?;
                model.save_config().map_err(Into::into)
            }
            IapyxCommand::Refresh => {
                model.refresh_state()?;
                model.save_config().map_err(Into::into)
            }
            IapyxCommand::Logs => {
                let mut table = table();
                table.set_titles(row!["id", "status"]);
                model
                    .fragment_logs()?
                    .iter()
                    .for_each(|(id, fragment_log)| {
                        let compact = match fragment_log.status() {
                            FragmentStatus::Pending => "pending".to_string(),
                            FragmentStatus::Rejected { reason } => format!("rejected: {reason}"),
                            FragmentStatus::InABlock { .. } => "in a block".to_string(),
                        };
                        table.add_row(row![id, compact]);
                    });
                table.printstd();
                Ok(())
            }
            IapyxCommand::Funds => {
                println!("{:#?}", model.funds()?);
                Ok(())
            }
            IapyxCommand::Statuses => {
                let mut table = table();
                table.set_titles(row!["index", "id", "status"]);
                model
                    .statuses()?
                    .iter()
                    .enumerate()
                    .for_each(|(idx, (id, status))| {
                        table.add_row(row![idx, id, format!("{:?}", status)]);
                    });
                table.printstd();
                Ok(())
            }
            IapyxCommand::Votes(votes) => votes.exec(model),
        }
    }
}

#[derive(Parser, Debug)]
pub struct Votes {
    /// Id of input vote plan
    #[clap(long = "vote-plan-id")]
    pub vote_plan_id: Option<String>,
    /// Index of vote plan
    #[clap(long = "vote-plan-index", conflicts_with = "vote-plan-id")]
    pub vote_plan_index: Option<usize>,
    /// Print title, otherwise only id would be print out
    #[clap(long = "print-details")]
    pub print_details: bool,
    #[clap(default_value = "direct", long)]
    pub voting_group: String,
}

impl Votes {
    pub fn exec(&self, model: CliController) -> Result<(), IapyxCommandError> {
        let vote_plan_id: Option<String> = {
            if let Some(index) = self.vote_plan_index {
                let funds = model.funds()?;
                Some(funds.chain_vote_plans[index].chain_voteplan_id.to_string())
            } else {
                self.vote_plan_id
                    .as_ref()
                    .map(|vote_plan_id| vote_plan_id.to_string())
            }
        };

        let mut table = table();

        let maybe_history_of_votes = if let Some(vote_plan_id) = vote_plan_id {
            let vote_plan_id_hash = Hash::from_str(&vote_plan_id)?;
            model.vote_plan_history(vote_plan_id_hash)?.map(|votes| {
                vec![AccountVotes {
                    vote_plan_id: vote_plan_id_hash,
                    votes,
                }]
            })
        } else {
            model.votes_history()?
        };

        if maybe_history_of_votes.is_none() {
            println!("No votes are cast so far");
            return Ok(());
        }

        if let Some(votes_history) = maybe_history_of_votes {
            if self.print_details {
                let proposals = model.proposals(&self.voting_group)?;
                table.set_titles(row!["vote plan", "proposals"]);

                votes_history
                    .iter()
                    .map(|x| {
                        let indexes_and_titles: Vec<(u8, String)> = x
                            .votes
                            .iter()
                            .map(|id| {
                                let proposal = proposals
                                    .iter()
                                    .find(|y| {
                                        *id == y.voteplan.chain_proposal_index as u8
                                            && y.voteplan.chain_voteplan_id
                                                == x.vote_plan_id.to_string()
                                    })
                                    .expect(
                                        "internal error: cannot find proposal for history of votes",
                                    );

                                (*id, proposal.proposal.proposal_title.clone())
                            })
                            .collect();

                        (x.vote_plan_id, indexes_and_titles)
                    })
                    .for_each(|(voteplan, proposals)| {
                        let mut inner_table = empty_table();
                        proposals.iter().for_each(|(id, title)| {
                            inner_table.add_row(row![id, title]);
                        });
                        table.add_row(row![voteplan, inner_table]);
                    });
            } else {
                table.set_titles(row!["vote plan", "proposals indexes"]);
                votes_history.iter().for_each(|x| {
                    let mut inner_table = empty_table();
                    x.votes.iter().for_each(|v| {
                        inner_table.add_row(row![v]);
                    });
                    table.add_row(row![x.vote_plan_id, inner_table]);
                });
            }
        };
        table.printstd();
        Ok(())
    }
}

#[derive(Parser, Debug)]
pub enum Vote {
    /// Send single vote
    Single(SingleVote),
    /// Send batch of votes
    Batch(BatchOfVotes),
}

impl Vote {
    pub fn exec(self, model: CliController) -> Result<(), IapyxCommandError> {
        match self {
            Self::Single(single) => single.exec(model),
            Self::Batch(batch) => batch.exec(model),
        }
    }
}

#[derive(Parser, Debug)]
pub struct SingleVote {
    /// Choice, usually 'yes' or 'no'
    #[clap(short = 'c', long = "choice")]
    pub choice: String,
    /// Proposal id of target proposal. It can be obtained from `iapyx proposals` command
    #[clap(short = 'i', long = "id")]
    pub proposal_id: String,
    /// Transaction expiry fixed time
    #[clap(long = "valid-until-fixed")]
    pub valid_until_fixed: Option<BlockDate>,
    /// Transaction expiry shifted time
    #[clap(long = "valid-until-shift", conflicts_with = "valid-until-fixed")]
    pub valid_until_shift: Option<BlockDate>,
    /// Pin
    #[clap(long, short)]
    pub pin: String,
    #[clap(default_value = "direct", long)]
    pub voting_group: String,
}

impl SingleVote {
    pub fn exec(self, mut model: CliController) -> Result<(), IapyxCommandError> {
        let proposals = model.proposals(&self.voting_group)?;

        let proposal = proposals
            .iter()
            .find(|x| x.chain_proposal_id_as_str() == self.proposal_id)
            .ok_or_else(|| IapyxCommandError::CannotFindProposal(self.proposal_id.clone()))?;
        let choice = proposal
            .proposal
            .chain_vote_options
            .0
            .get(&self.choice)
            .ok_or_else(|| IapyxCommandError::WrongChoice(self.choice.clone()))?;
        model.vote(proposal, Choice::new(*choice), &self.pin)?;
        model.save_config()?;
        Ok(())
    }
}

#[derive(Parser, Debug)]
pub struct BatchOfVotes {
    /// Choice, usually 'yes' or 'no'
    #[clap(short = 'c', long = "choices")]
    pub choices: String,
    /// Transaction expiry time
    #[clap(long)]
    pub valid_until_fixed: Option<BlockDate>,
    /// Transaction expiry time
    #[clap(long, conflicts_with = "valid-until-fixed")]
    pub valid_until_shift: Option<BlockDate>,
    /// Pin
    #[clap(long, short)]
    pub pin: String,
    #[clap(default_value = "direct", long)]
    pub voting_group: String,
}

impl BatchOfVotes {
    pub fn exec(self, mut model: CliController) -> Result<(), IapyxCommandError> {
        let choices = self.zip_into_batch_input_data(
            serde_json::from_str(&self.choices)?,
            model.proposals(&self.voting_group)?,
        )?;
        model.votes_batch(choices.iter().map(|(p, c)| (p, *c)).collect(), &self.pin)?;
        model.save_config()?;
        Ok(())
    }

    fn zip_into_batch_input_data(
        &self,
        choices: HashMap<String, String>,
        proposals: Vec<FullProposalInfo>,
    ) -> Result<Vec<(FullProposalInfo, Choice)>, IapyxCommandError> {
        let mut result = Vec::new();

        for (proposal_id, choice) in choices {
            let proposal = proposals
                .iter()
                .find(|x| x.chain_proposal_id_as_str() == *proposal_id)
                .ok_or_else(|| IapyxCommandError::CannotFindProposal(proposal_id.clone()))?;

            let choice = proposal
                .proposal
                .chain_vote_options
                .0
                .get(&choice)
                .ok_or_else(|| IapyxCommandError::WrongChoice(choice.clone()))?;

            result.push((proposal.clone(), Choice::new(*choice)));
        }
        Ok(result)
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum IapyxCommandError {
    #[error(transparent)]
    ControllerError(#[from] ControllerError),
    #[error(transparent)]
    Inner(#[from] thor::cli::Error),
    #[error(transparent)]
    CannotParseChoicesString(#[from] serde_json::Error),
    #[error("wrong choice: {0}")]
    WrongChoice(String),
    #[error("cannot find proposal: {0}")]
    CannotFindProposal(String),
    #[error(transparent)]
    ControllerBuilder(#[from] ControllerBuilderError),
    #[error(transparent)]
    Hash(#[from] chain_crypto::hash::Error),
    #[error(transparent)]
    Image(#[from] image::ImageError),
    #[error(transparent)]
    Controller(#[from] super::Error),
    #[error(transparent)]
    Bech32(#[from] bech32::Error),
    #[error(transparent)]
    Valgrind(#[from] valgrind::Error),
    #[error(transparent)]
    Config(#[from] thor::cli::ConfigError),
    #[error(transparent)]
    Backend(#[from] RestError),
    #[error(transparent)]
    KeyQrCode(#[from] KeyQrCodeError),
    #[error(transparent)]
    Key(#[from] jcli_lib::key::Error),
    #[error(transparent)]
    Read(#[from] chain_core::property::ReadError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Parsing command line failed")]
    Cli(#[from] CliParserError),
    #[error("Invalid regex")]
    Regex(#[from] regex::Error),
}

#[derive(Parser, Debug)]
pub struct Connect {
    /// Backend address. For example `https://catalyst.io/api`
    #[clap(name = "ADDRESS")]
    pub address: String,
    /// Uses https for sending fragments
    #[clap(short = 's', long = "https")]
    pub use_https: bool,
    /// Printing additional information
    #[clap(short = 'd', long = "enable-debug")]
    pub enable_debug: bool,
}

#[derive(Error, Debug)]
pub enum CliParserError {
    #[error("incorrect format of url. Tip: http(s)://address:port is acceptable")]
    IncorrectFormatOfAddressUrl,
}

impl Connect {
    pub fn exec(&self, mut controller: CliController) -> Result<(), IapyxCommandError> {
        Url::parse(&self.address).map_err(|_| CliParserError::IncorrectFormatOfAddressUrl)?;

        controller.update_connection(Connection {
            address: self.address.clone(),
            https: self.use_https,
            debug: self.enable_debug,
        });
        if controller.check_connection().is_ok() {
            println!("Connection succesfull.");
            controller.save_config().map_err(Into::into)
        } else {
            eprintln!("Connection unsuccesfull.");
            Ok(())
        }
    }
}

#[derive(Parser, Debug)]
pub struct Proposals {
    /// Limit output entries by text
    #[clap(short, long)]
    pub regex: Option<String>,
    #[clap(short, long)]
    pub sort: Option<String>,
    /// Limit output entries
    #[clap(short, long)]
    pub limit: Option<usize>,
    #[clap(default_value = "direct", long)]
    pub voting_group: String,
}
impl Proposals {
    pub fn exec(self, model: CliController) -> Result<(), IapyxCommandError> {
        let mut table = table();
        table.set_titles(row!["index", "title", "summary", "proposal hash",]);

        for (id, proposal) in model.proposals(&self.voting_group)?.iter().enumerate() {
            if let Some(limit) = self.limit {
                if id >= limit {
                    break;
                }
            }

            if let Some(regex) = &self.regex {
                use regex::Regex;
                let re = Regex::new(regex)?;

                if !re.is_match(&proposal.proposal.proposal_title) {
                    continue;
                }
            }

            table.add_row(row![
                (id + 1),
                proposal.proposal.proposal_title,
                proposal.proposal.proposal_summary,
                proposal.chain_proposal_id_as_str(),
            ]);
        }
        table.printstd();
        Ok(())
    }
}
#[derive(Parser, Debug)]
pub enum Wallets {
    /// Recover wallet funds from mnemonic
    Use {
        #[clap(name = "ALIAS")]
        alias: Alias,
    },
    /// Recover wallet funds from qr code
    Import {
        #[clap(short, long)]
        alias: Alias,

        #[clap(subcommand)]
        cmd: WalletAddSubcommand,
    },
    /// Delete wallet with alias
    Delete {
        #[clap(name = "ALIAS")]
        alias: Alias,
    },
    /// List already imported wallets
    List,
}

#[derive(Parser, Debug)]
pub enum WalletAddSubcommand {
    /// Recover wallet funds from mnemonic
    Secret {
        /// Path to secret file
        #[clap(name = "SECRET")]
        secret: PathBuf,

        /// Pin to protect you wallet.
        #[clap(short, long)]
        pin: String,

        /// If true testing discrimination is used, otherwise production
        #[clap(short, long)]
        testing: bool,
    },
    /// Recover wallet funds from qr code
    QR {
        /// Path to qr file
        #[clap(name = "QR")]
        qr: PathBuf,

        /// Pin to protect you wallet.
        #[clap(short, long)]
        pin: String,

        /// If true testing discrimination is used, otherwise production
        #[clap(short, long)]
        testing: bool,
    },
    /// recover wallet funds from hash
    Hash {
        /// Path to file with payload
        #[clap(name = "Hash")]
        hash: PathBuf,

        /// Pin to protect you wallet.
        #[clap(short, long)]
        pin: String,

        /// If true testing discrimination is used, otherwise production
        #[clap(short, long)]
        testing: bool,
    },
}

impl WalletAddSubcommand {
    pub fn add_wallet(
        self,
        mut controller: CliController,
        alias: Alias,
    ) -> Result<(), IapyxCommandError> {
        match self {
            Self::Secret {
                secret,
                pin,
                testing,
            } => {
                let (_, data, _) = read_bech32(Some(&secret))?;
                controller
                    .wallets_mut()
                    .add_wallet(alias, testing, data, &pin)?
            }
            Self::QR { qr, pin, testing } => {
                let img = image::open(qr)?;
                let bytes: Vec<u8> = pin.chars().map(|x| x.to_digit(10).unwrap() as u8).collect();
                let secret = KeyQrCode::decode(img, &bytes)?
                    .first()
                    .unwrap()
                    .clone()
                    .leak_secret();

                controller
                    .wallets_mut()
                    .add_wallet(alias, testing, secret.to_base32(), &pin)?
            }
            Self::Hash { hash, pin, testing } => {
                let bytes: Vec<u8> = pin.chars().map(|x| x.to_digit(10).unwrap() as u8).collect();
                let secret = decode(jortestkit::file::read_file(hash)?, &bytes)
                    .unwrap()
                    .leak_secret();
                controller
                    .wallets_mut()
                    .add_wallet(alias, testing, secret.to_base32(), &pin)?
            }
        };
        controller.wallets().save_config().map_err(Into::into)
    }
}

impl Wallets {
    pub fn exec(self, mut model: CliController) -> Result<(), IapyxCommandError> {
        match self {
            Self::Use { alias } => {
                model.wallets_mut().set_default_alias(alias)?;
                model.wallets().save_config().map_err(Into::into)
            }
            Self::Import { alias, cmd } => cmd.add_wallet(model, alias),
            Self::Delete { alias } => {
                model.wallets_mut().remove_wallet(alias)?;
                model.wallets().save_config().map_err(Into::into)
            }
            Self::List => {
                let mut table = table();
                table.set_titles(row!["in use", "index", "alias", "public key"]);
                model
                    .wallets()
                    .iter()
                    .enumerate()
                    .for_each(|(idx, (alias, wallet))| {
                        table.add_row(if Some(alias) == model.wallets().default_alias() {
                            row!["  ->  ", idx + 1, alias, wallet.public_key]
                        } else {
                            row!["", idx + 1, alias, wallet.public_key]
                        });
                    });
                table.printstd();
                Ok(())
            }
        }
    }
}

fn table() -> Table {
    let mut table = Table::new();
    table.set_format(
        FormatBuilder::new()
            .column_separator(' ')
            .borders(' ')
            .separators(
                &[LinePosition::Title],
                LineSeparator::new('─', ' ', ' ', ' '),
            )
            .padding(1, 1)
            .build(),
    );
    table
}

fn empty_table() -> Table {
    let mut inner_table = Table::new();
    inner_table.set_format(
        FormatBuilder::new()
            .column_separator(' ')
            .borders(' ')
            .padding(1, 1)
            .build(),
    );
    inner_table
}
