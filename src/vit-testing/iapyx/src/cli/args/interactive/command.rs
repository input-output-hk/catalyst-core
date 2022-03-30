use crate::cli::args::interactive::CliController;
use bech32::ToBase32;
use catalyst_toolbox::kedqr::decode;
use catalyst_toolbox::kedqr::KeyQrCode;
use catalyst_toolbox::kedqr::KeyQrCodeError;
use chain_impl_mockchain::block::BlockDate;
use jcli_lib::key::read_bech32;
use jormungandr_automation::jormungandr::RestError;
use jormungandr_lib::crypto::hash::Hash;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;
use thiserror::Error;
use thor::cli::{Alias, Connection};
use valgrind::{Proposal, ProposalExtension};
use wallet_core::Choice;
#[derive(StructOpt, Debug)]
pub enum IapyxCommand {
    /// connect to backend
    Connect(Connect),
    /// get Address
    Address,
    /// printout proposals
    Proposals(Proposals),
    /// Prints wallet status
    Status,
    /// clear transaction
    ClearTx,
    /// confirms transaction
    ConfirmTx,
    /// Prints wallets, nodes which can be used. Draw topology
    Refresh,
    Logs,
    Funds,
    Statuses,
    Vote(Vote),
    Votes(Votes),
    PendingTransactions,
    Wallets(Wallets),
}

const DELIMITER: &str = "===================";

fn print_delim() {
    println!("{}", DELIMITER);
}

impl IapyxCommand {
    pub fn exec(self, mut model: CliController) -> Result<(), IapyxCommandError> {
        match self {
            IapyxCommand::Wallets(wallets) => wallets.exec(model),
            IapyxCommand::Connect(connect) => connect.exec(model),
            IapyxCommand::Proposals(proposals) => proposals.exec(model),
            IapyxCommand::Address => {
                let wallet = model.wallets().wallet()?;
                println!("Address: {}", wallet.address_readable()?);
                println!("Account id: {}", wallet.id()?);
                Ok(())
            }
            IapyxCommand::Status => {
                let account_state = model.account_state()?;
                print_delim();
                println!("- Delegation: {:?}", account_state.delegation());
                println!("- Value: {}", account_state.value());
                println!("- Spending counters: {:?}", account_state.counters());
                println!("- Rewards: {:?}", account_state.last_rewards());
                println!("- Tokens: {:?}", account_state.tokens());
                print_delim();
                Ok(())
            }
            IapyxCommand::PendingTransactions => {
                print_delim();
                for (idx, fragment_ids) in model.wallets().wallet()?.pending_tx.iter().enumerate() {
                    println!("{}. {}", (idx + 1), fragment_ids);
                }
                print_delim();
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
                println!("{:#?}", model.fragment_logs()?);
                Ok(())
            }
            IapyxCommand::Funds => {
                println!("{:#?}", model.funds()?);
                Ok(())
            }
            IapyxCommand::Statuses => {
                print_delim();
                for (idx, (id, status)) in model.statuses()?.iter().enumerate() {
                    println!("{}. {} -> {:#?}", idx, id, status);
                }
                print_delim();
                Ok(())
            }
            IapyxCommand::Votes(votes) => votes.exec(model),
        }
    }
}

#[derive(StructOpt, Debug)]
pub struct Votes {
    #[structopt(long = "vote-plan-id")]
    pub vote_plan_id: Option<String>,
    #[structopt(long = "vote-plan-index", conflicts_with = "vote-plan-id")]
    pub vote_plan_index: Option<usize>,
    #[structopt(long = "print-title")]
    pub print_proposal_title: bool,
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

        print_delim();
        match vote_plan_id {
            Some(vote_plan_id) => {
                let vote_plan_id_hash = Hash::from_str(&vote_plan_id)?;
                if self.print_proposal_title {
                    let history = model.vote_plan_history(vote_plan_id_hash)?;
                    let proposals = model.proposals()?;

                    if let Some(history) = history {
                        let history: Vec<String> = history
                            .iter()
                            .map(|x| {
                                proposals
                                    .iter()
                                    .find(|y| {
                                        y.chain_proposal_index as u8 == *x
                                            && y.chain_voteplan_id == vote_plan_id
                                    })
                                    .unwrap()
                            })
                            .map(|p| p.proposal_title.clone())
                            .collect();
                        println!("{:#?}", history);
                    } else {
                        println!("Vote plan not found",);
                    }
                } else {
                    println!("{:#?}", model.vote_plan_history(vote_plan_id_hash)?);
                }
            }
            None => {
                if self.print_proposal_title {
                    let history = model.votes_history()?;
                    let proposals = model.proposals()?;

                    if let Some(history) = history {
                        let history: Vec<String> = history
                            .iter()
                            .map(|x| {
                                proposals
                                    .iter()
                                    .find(|y| {
                                        x.votes.contains(&(y.chain_proposal_index as u8))
                                            && y.chain_voteplan_id == x.vote_plan_id.to_string()
                                    })
                                    .unwrap()
                            })
                            .map(|p| p.proposal_title.clone())
                            .collect();
                        println!("{:#?}", history)
                    } else {
                        println!("Cannot find any voteplan");
                    }
                } else {
                    println!("{:#?}", model.votes_history()?);
                }
            }
        };
        print_delim();
        Ok(())
    }
}

#[derive(StructOpt, Debug)]
pub enum Vote {
    Single(SingleVote),
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

#[derive(StructOpt, Debug)]
pub struct SingleVote {
    /// choice
    #[structopt(short = "c", long = "choice")]
    pub choice: String,
    /// chain proposal id
    #[structopt(short = "i", long = "id")]
    pub proposal_id: String,

    // transaction expiry fixed  time
    #[structopt(long = "valid-until-fixed")]
    pub valid_until_fixed: Option<BlockDate>,

    // transaction expiry shifted time
    #[structopt(long = "valid-until-shift", conflicts_with = "valid-until-fixed")]
    pub valid_until_shift: Option<BlockDate>,

    // pin
    #[structopt(long, short)]
    pub pin: String,
}

impl SingleVote {
    pub fn exec(self, mut model: CliController) -> Result<(), IapyxCommandError> {
        let proposals = model.proposals()?;
        /*   let block_date_generator = expiry::from_block_or_shift(
            self.valid_until_fixed,
            self.valid_until_shift,
            &model.backend_client()?.settings()?,
        );*/

        let proposal = proposals
            .iter()
            .find(|x| x.chain_proposal_id_as_str() == self.proposal_id)
            .ok_or_else(|| IapyxCommandError::CannotFindProposal(self.proposal_id.clone()))?;
        let choice = proposal
            .chain_vote_options
            .0
            .get(&self.choice)
            .ok_or_else(|| IapyxCommandError::WrongChoice(self.choice.clone()))?;
        model.vote(proposal, Choice::new(*choice), &self.pin)?;
        model.save_config()?;
        Ok(())
    }
}

#[derive(StructOpt, Debug)]
pub struct BatchOfVotes {
    /// choice
    #[structopt(short = "c", long = "choices")]
    pub choices: String,

    // transaction expiry time
    #[structopt(long)]
    pub valid_until_fixed: Option<BlockDate>,

    // transaction expiry time
    #[structopt(long, conflicts_with = "valid-until-fixed")]
    pub valid_until_shift: Option<BlockDate>,

    // pin
    #[structopt(long, short)]
    pub pin: String,
}

impl BatchOfVotes {
    pub fn exec(self, mut model: CliController) -> Result<(), IapyxCommandError> {
        let choices = self
            .zip_into_batch_input_data(serde_json::from_str(&self.choices)?, model.proposals()?)?;
        model.votes_batch(choices.iter().map(|(p, c)| (p, *c)).collect(), &self.pin)?;
        model.save_config()?;
        Ok(())
    }

    fn zip_into_batch_input_data(
        &self,
        choices: HashMap<String, String>,
        proposals: Vec<Proposal>,
    ) -> Result<Vec<(Proposal, Choice)>, IapyxCommandError> {
        let mut result = Vec::new();

        for (proposal_id, choice) in choices {
            let proposal = proposals
                .iter()
                .find(|x| x.chain_proposal_id_as_str() == *proposal_id)
                .ok_or_else(|| IapyxCommandError::CannotFindProposal(proposal_id.clone()))?;

            let choice = proposal
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
    #[error("{0}")]
    GeneralError(String),
    #[error(transparent)]
    ControllerError(#[from] crate::controller::ControllerError),
    #[error(transparent)]
    Inner(#[from] thor::cli::Error),
    #[error(transparent)]
    CannotParseChoicesString(#[from] serde_json::Error),
    #[error("no valid until defined")]
    NoValidUntilDefined,
    #[error("wallet not recovered or generated")]
    WalletNotRecovered,
    #[error("wrong choice: {0}")]
    WrongChoice(String),
    #[error("cannot find proposal: {0}")]
    CannotFindProposal(String),
    #[error(transparent)]
    ControllerBuilder(#[from] crate::controller::ControllerBuilderError),
    #[error(transparent)]
    Hash(#[from] chain_crypto::hash::Error),
    #[error(transparent)]
    Image(#[from] image::ImageError),
    #[error(transparent)]
    Controller(#[from] crate::cli::args::interactive::Error),
    #[error("there is no default alias defined in config nor provided as argument")]
    AliasNotDefined,
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
}

#[derive(StructOpt, Debug)]
pub struct Connect {
    #[structopt(name = "ADDRESS")]
    pub address: String,

    /// uses https for sending fragments
    #[structopt(short = "s", long = "https")]
    pub use_https: bool,

    /// uses https for sending fragments
    #[structopt(short = "d", long = "enable-debug")]
    pub enable_debug: bool,
}

impl Connect {
    pub fn exec(&self, mut controller: CliController) -> Result<(), IapyxCommandError> {
        controller.update_connection(Connection {
            address: self.address.clone(),
            https: self.use_https,
            debug: self.enable_debug,
        });
        controller.check_connection()?;
        controller.save_config().map_err(Into::into)
    }
}

#[derive(StructOpt, Debug)]
pub struct Proposals {
    /// show only ids
    #[structopt(short = "i")]
    pub only_ids: bool,

    /// show only ids
    #[structopt(short, long)]
    pub limit: Option<usize>,
}
impl Proposals {
    pub fn exec(self, model: CliController) -> Result<(), IapyxCommandError> {
        print_delim();
        for (id, proposal) in model.proposals()?.iter().enumerate() {
            if let Some(limit) = self.limit {
                if id >= limit {
                    break;
                }
            }

            if self.only_ids {
                println!("{}", proposal.chain_proposal_id_as_str());
            } else {
                println!(
                    "{}. {} [{}] {}",
                    (id + 1),
                    proposal.chain_proposal_id_as_str(),
                    proposal.proposal_title,
                    proposal.proposal_summary
                );
                println!("{:#?}", proposal.chain_vote_options.0);
            }
        }
        print_delim();
        Ok(())
    }
}
#[derive(StructOpt, Debug)]
pub enum Wallets {
    /// recover wallet funds from mnemonic
    Use {
        #[structopt(name = "ALIAS")]
        alias: Alias,
    },
    /// recover wallet funds from qr code
    Import {
        #[structopt(short, long)]
        alias: Alias,

        #[structopt(subcommand)] // Note that we mark a field as a subcommand
        cmd: WalletAddSubcommand,
    },
    Delete {
        #[structopt(name = "ALIAS")]
        alias: Alias,
    },
    List,
}

#[derive(StructOpt, Debug)]
pub enum WalletAddSubcommand {
    /// recover wallet funds from mnemonic
    Secret {
        #[structopt(name = "SECRET")]
        secret: PathBuf,

        #[structopt(short, long)]
        password: String,

        #[structopt(short, long)]
        testing: bool,
    },
    /// recover wallet funds from qr code
    QR {
        #[structopt(name = "QR")]
        qr: PathBuf,

        #[structopt(short, long)]
        pin: String,

        #[structopt(short, long)]
        testing: bool,
    },
    /// recover wallet funds from hash
    Hash {
        #[structopt(name = "Hash")]
        hash: PathBuf,

        #[structopt(short, long)]
        pin: String,

        #[structopt(short, long)]
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
                password,
                testing,
            } => {
                let (_, data, _) = read_bech32(Some(&secret))?;
                controller
                    .wallets_mut()
                    .add_wallet(alias, testing, data, &password)?
            }
            Self::QR { qr, pin, testing } => {
                let img = image::open(qr)?;
                let bytes: Vec<u8> = pin.chars().map(|x| x.to_digit(10).unwrap() as u8).collect();
                let secret = KeyQrCode::decode(img, &bytes)?
                    .get(0)
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
                for (idx, (alias, wallet)) in model.wallets().iter().enumerate() {
                    if Some(alias) == model.wallets().default_alias() {
                        println!("[Default]{}.\t{}\t{}", idx + 1, alias, wallet.public_key);
                    } else {
                        println!("{}.\t{}\t{}", idx + 1, alias, wallet.public_key);
                    }
                }
                Ok(())
            }
        }
    }
}
