use super::WalletState;
use crate::cli::args::interactive::UserInteractionContoller;
use crate::utils::valid_until::ValidUntil;
use crate::Controller;
use crate::Proposal;
use crate::WalletBackend;
use bip39::Type;
use chain_addr::{AddressReadable, Discrimination};
use chain_impl_mockchain::block::BlockDate;
use jormungandr_testing_utils::testing::node::RestSettings;
use std::collections::HashMap;
use std::path::PathBuf;
use structopt::{clap::AppSettings, StructOpt};
use thiserror::Error;
use wallet_core::Choice;

#[derive(StructOpt, Debug)]
#[structopt(setting = AppSettings::NoBinaryName)]
pub enum IapyxCommand {
    /// recover wallet funds from mnemonic
    Recover(Recover),
    /// generate new wallet
    Generate(Generate),
    /// connect to backend
    Connect(Connect),
    /// confirms transaction
    ConfirmTx,
    Value,
    /// Prints wallets, nodes which can be used. Draw topology
    Status,
    /// Prints wallets, nodes which can be used. Draw topology
    Refresh,
    /// get Address
    Address(Address),
    Logs,
    /// Exit interactive mode
    Exit,
    Proposals(Proposals),
    Vote(Vote),
    Votes,
    PendingTransactions,
}

const DELIMITER: &str = "===================";

fn print_delim() {
    println!("{}", DELIMITER);
}

impl IapyxCommand {
    pub fn exec(&self, model: &mut UserInteractionContoller) -> Result<(), IapyxCommandError> {
        match self {
            IapyxCommand::PendingTransactions => {
                if let Some(controller) = model.controller.as_mut() {
                    let fragment_ids = controller
                        .pending_transactions()
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>();
                    print_delim();
                    for (id, fragment_ids) in fragment_ids.iter().enumerate() {
                        println!("{}. {}", (id + 1), fragment_ids);
                    }
                    print_delim();
                    return Ok(());
                }
                Err(IapyxCommandError::WalletNotRecovered)
            }
            IapyxCommand::Votes => {
                if let Some(controller) = model.controller.as_mut() {
                    print_delim();
                    for (id, vote) in controller.active_votes()?.iter().enumerate() {
                        println!("{}. {}", (id + 1), vote);
                    }
                    print_delim();
                    return Ok(());
                }
                Err(IapyxCommandError::WalletNotRecovered)
            }
            IapyxCommand::Proposals(proposals) => {
                if let Some(controller) = model.controller.as_mut() {
                    print_delim();
                    for (id, proposal) in controller.get_proposals()?.iter().enumerate() {
                        if proposals.only_ids {
                            println!("{}", proposal.chain_proposal_id_as_str());
                        } else {
                            println!(
                                "{}. #{} [{}] {}",
                                (id + 1),
                                proposal.chain_proposal_id_as_str(),
                                proposal.proposal_title,
                                proposal.proposal_summary
                            );
                            println!("{:#?}", proposal.chain_vote_options.0);
                        }
                    }
                    print_delim();
                    return Ok(());
                }
                Err(IapyxCommandError::WalletNotRecovered)
            }
            IapyxCommand::Vote(vote) => vote.exec(model),
            IapyxCommand::ConfirmTx => {
                if let Some(controller) = model.controller.as_mut() {
                    controller.confirm_all_transactions();
                    return Ok(());
                }
                Err(IapyxCommandError::WalletNotRecovered)
            }
            IapyxCommand::Recover(recover) => recover.exec(model),
            IapyxCommand::Exit => Ok(()),
            IapyxCommand::Generate(generate) => generate.exec(model),
            IapyxCommand::Connect(connect) => connect.exec(model),
            IapyxCommand::Value => {
                if let Some(controller) = model.controller.as_mut() {
                    println!("Total Value: {}", controller.total_value());
                    return Ok(());
                }
                Err(IapyxCommandError::WalletNotRecovered)
            }
            IapyxCommand::Status => {
                if let Some(controller) = model.controller.as_ref() {
                    let account_state = controller.get_account_state()?;
                    print_delim();
                    println!("- Delegation: {:?}", account_state.delegation());
                    println!("- Value: {}", account_state.value());
                    println!("- Spending counter: {}", account_state.counter());
                    println!("- Rewards: {:?}", account_state.last_rewards());
                    print_delim();
                    return Ok(());
                }
                Err(IapyxCommandError::WalletNotRecovered)
            }
            IapyxCommand::Refresh => {
                if let Some(controller) = model.controller.as_mut() {
                    controller.refresh_state()?;
                    return Ok(());
                }
                Err(IapyxCommandError::WalletNotRecovered)
            }
            IapyxCommand::Address(address) => address.exec(model),
            IapyxCommand::Logs => {
                if let Some(controller) = model.controller.as_mut() {
                    println!("{:#?}", controller.fragment_logs());
                    return Ok(());
                }
                Err(IapyxCommandError::WalletNotRecovered)
            }
        }
    }
}

#[derive(StructOpt, Debug)]
pub struct Address {
    /// blocks execution until fragment is in block
    #[structopt(short = "t", long = "testing")]
    pub testing: bool,
}

impl Address {
    pub fn exec(&self, model: &mut UserInteractionContoller) -> Result<(), IapyxCommandError> {
        if let Some(controller) = model.controller.as_mut() {
            let (prefix, discrimination) = {
                if self.testing {
                    ("ta", Discrimination::Test)
                } else {
                    ("ca", Discrimination::Production)
                }
            };
            let address =
                AddressReadable::from_address(prefix, &controller.account(discrimination));
            println!("{}", address.to_string());
            return Ok(());
        }
        Err(IapyxCommandError::WalletNotRecovered)
    }
}

#[derive(StructOpt, Debug)]
pub enum Vote {
    Single(SingleVote),
    Batch(BatchOfVotes),
}

impl Vote {
    pub fn exec(&self, model: &mut UserInteractionContoller) -> Result<(), IapyxCommandError> {
        match self {
            Self::Single(single) => single.exec(model),
            Self::Batch(batch) => batch.exec(model),
        }
    }
}

#[derive(StructOpt, Debug)]
pub struct Proposals {
    /// choice
    #[structopt(short = "i")]
    pub only_ids: bool,
}

#[derive(StructOpt, Debug)]
pub struct SingleVote {
    /// choice
    #[structopt(short = "c", long = "choice")]
    pub choice: String,
    /// chain proposal id
    #[structopt(short = "p", long = "id")]
    pub proposal_id: String,

    // transaction expiry time
    #[structopt(long)]
    pub valid_until_fixed: Option<BlockDate>,

    // transaction expiry time
    #[structopt(long, conflicts_with = "valid-until-fixed")]
    pub valid_until_shift: Option<u32>,
}

impl SingleVote {
    pub fn exec(&self, model: &mut UserInteractionContoller) -> Result<(), IapyxCommandError> {
        if let Some(controller) = model.controller.as_mut() {
            let proposals = controller.get_proposals()?;
            let valid_until =
                &ValidUntil::from_block_or_shift(self.valid_until_fixed, self.valid_until_shift)
                    .ok_or(IapyxCommandError::NoValidUntilDefined)?;

            let proposal = proposals
                .iter()
                .find(|x| x.chain_proposal_id_as_str() == self.proposal_id)
                .ok_or_else(|| IapyxCommandError::CannotFindProposal(self.proposal_id.clone()))?;
            let choice = proposal
                .chain_vote_options
                .0
                .get(&self.choice)
                .ok_or_else(|| IapyxCommandError::WrongChoice(self.choice.clone()))?;
            controller.vote(proposal, Choice::new(*choice), valid_until)?;
            return Ok(());
        }
        Err(IapyxCommandError::WalletNotRecovered)
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
    pub valid_until_shift: Option<u32>,
}

impl BatchOfVotes {
    pub fn exec(&self, model: &mut UserInteractionContoller) -> Result<(), IapyxCommandError> {
        if let Some(controller) = model.controller.as_mut() {
            let valid_until =
                &ValidUntil::from_block_or_shift(self.valid_until_fixed, self.valid_until_shift)
                    .ok_or(IapyxCommandError::NoValidUntilDefined)?;

            let choices = self.zip_into_batch_input_data(
                serde_json::from_str(&self.choices)?,
                controller.get_proposals()?,
            )?;
            controller.votes_batch(choices.iter().map(|(p, c)| (p, *c)).collect(), valid_until)?;
            return Ok(());
        }
        Err(IapyxCommandError::WalletNotRecovered)
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

#[derive(StructOpt, Debug)]
pub struct Connect {
    #[structopt(name = "ADDRESS")]
    pub address: String,

    /// uses https for sending fragments
    #[structopt(short = "s", long = "use-https")]
    pub use_https: bool,

    /// uses https for sending fragments
    #[structopt(short = "d", long = "enable-debug")]
    pub enable_debug: bool,
}

impl Connect {
    pub fn exec(&self, model: &mut UserInteractionContoller) -> Result<(), IapyxCommandError> {
        let settings = RestSettings {
            use_https: self.use_https,
            enable_debug: self.enable_debug,
            ..Default::default()
        };

        if let Some(controller) = model.controller.as_mut() {
            controller.switch_backend(self.address.clone(), settings);
            return Ok(());
        }

        model.backend_address = self.address.clone();
        model.settings = settings;
        Ok(())
    }
}

#[derive(StructOpt, Debug)]
pub enum Recover {
    /// recover wallet funds from mnemonic
    Mnemonics(RecoverFromMnemonics),
    /// recover wallet funds from qr code
    Qr(RecoverFromQr),
    /// recover wallet funds from private key
    Secret(RecoverFromSecretKey),
}

impl Recover {
    pub fn exec(&self, model: &mut UserInteractionContoller) -> Result<(), IapyxCommandError> {
        match self {
            Recover::Mnemonics(mnemonics) => mnemonics.exec(model),
            Recover::Qr(qr) => qr.exec(model),
            Recover::Secret(sk) => sk.exec(model),
        }
    }
}

#[derive(StructOpt, Debug)]
pub struct RecoverFromSecretKey {
    #[structopt(name = "INPUT")]
    pub input: PathBuf,
}

impl RecoverFromSecretKey {
    pub fn exec(&self, model: &mut UserInteractionContoller) -> Result<(), IapyxCommandError> {
        let wallet_backend =
            WalletBackend::new(model.backend_address.clone(), model.settings.clone());

        model.controller = Some(Controller::recover_from_sk(wallet_backend, &self.input)?);
        model.state = WalletState::Recovered;
        Ok(())
    }
}

#[derive(StructOpt, Debug)]
pub struct RecoverFromQr {
    #[structopt(short = "q", long = "qr")]
    pub qr_code: PathBuf,

    #[structopt(short = "p", long = "password")]
    pub password: String,
}

impl RecoverFromQr {
    pub fn exec(&self, model: &mut UserInteractionContoller) -> Result<(), IapyxCommandError> {
        model.controller = Some(Controller::recover_from_qr(
            model.backend_address.clone(),
            &self.qr_code,
            &self.password,
            model.settings.clone(),
        )?);
        model.state = WalletState::Recovered;
        Ok(())
    }
}

#[derive(StructOpt, Debug)]
pub struct RecoverFromMnemonics {
    #[structopt(short = "m", long = "mnemonics")]
    pub mnemonics: Vec<String>,
}

impl RecoverFromMnemonics {
    pub fn exec(&self, model: &mut UserInteractionContoller) -> Result<(), IapyxCommandError> {
        model.controller = Some(Controller::recover(
            model.backend_address.clone(),
            &self.mnemonics.join(" "),
            &[],
            model.settings.clone(),
        )?);
        model.state = WalletState::Recovered;
        Ok(())
    }
}

#[derive(StructOpt, Debug)]
pub struct Generate {
    /// Words count
    #[structopt(short = "w", long = "words")]
    pub count: usize,
}

impl Generate {
    pub fn exec(&self, model: &mut UserInteractionContoller) -> Result<(), IapyxCommandError> {
        model.controller = Some(Controller::generate(
            model.backend_address.clone(),
            Type::from_word_count(self.count)?,
            model.settings.clone(),
        )?);
        model.state = WalletState::Generated;
        Ok(())
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum IapyxCommandError {
    #[error("{0}")]
    GeneralError(String),
    #[error(transparent)]
    ControllerError(#[from] crate::controller::ControllerError),
    #[error("wrong word count for generating wallet")]
    GenerateWalletError(#[from] bip39::Error),
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
}
