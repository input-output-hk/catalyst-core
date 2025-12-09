mod command;

use bech32::u5;
use bech32::FromBase32;
use chain_impl_mockchain::block::BlockDate;
use chain_impl_mockchain::fragment::FragmentId;
use cocoon::Cocoon;
pub use command::{IapyxCommand, IapyxCommandError};
use iapyx::Controller;
use iapyx::Wallet;
use jormungandr_automation::jormungandr::RestError;
use jormungandr_lib::interfaces::AccountState;
use jormungandr_lib::interfaces::AccountVotes;
use jormungandr_lib::interfaces::FragmentLog;
use jormungandr_lib::interfaces::FragmentStatus;
use jormungandr_lib::interfaces::VotePlanId;
use std::collections::HashMap;
use thiserror::Error;
use thor::cli::CliController as ThorCliController;
use thor::cli::ConfigManager;
use thor::cli::Connection;
use thor::cli::WalletController;
use thor::BlockDateGenerator;
use valgrind::Fund;
use valgrind::SettingsExtensions;
use valgrind::ValgrindClient;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
use wallet_core::Choice;
use wallet_core::Value;

pub struct CliController {
    pub inner: ThorCliController,
    pub backend_client: ValgrindClient,
}

impl CliController {
    pub fn new() -> Result<Self, Error> {
        let config_manager = ConfigManager::new(env!("CARGO_PKG_NAME"));
        let config = config_manager.read_config()?;
        let backend_client = ValgrindClient::new(
            config.connection.address.clone(),
            config.connection.clone().into(),
        )?;

        Ok(Self {
            inner: ThorCliController::new_from_client(config.connection.into(), config_manager)?,
            backend_client,
        })
    }

    pub fn update_connection(&mut self, connection: Connection) {
        self.inner.update_connection(connection);
    }

    pub fn check_connection(&self) -> Result<(), Error> {
        self.inner.check_connection().map_err(Into::into)
    }

    pub fn save_config(&self) -> Result<(), Error> {
        self.inner.save_config().map_err(Into::into)
    }

    pub fn wallets(&self) -> &WalletController {
        self.inner.wallets()
    }

    pub fn wallets_mut(&mut self) -> &mut WalletController {
        self.inner.wallets_mut()
    }

    pub fn refresh_state(&mut self) -> Result<(), Error> {
        self.inner.refresh_state().map_err(Into::into)
    }

    pub fn clear_txs(&mut self) -> Result<(), Error> {
        self.inner.clear_txs().map_err(Into::into)
    }

    pub fn confirm_txs(&mut self) -> Result<(), Error> {
        self.inner.confirm_tx().map_err(Into::into)
    }

    pub fn account_state(&self) -> Result<AccountState, Error> {
        self.inner.account_state().map_err(Into::into)
    }

    fn controller(&self, password: &str) -> Result<Controller, Error> {
        let template = self.inner.wallets().wallet()?;
        let contents = std::fs::read(&template.secret_file)?;
        let mut cocoon = Cocoon::new(password.as_bytes());

        let unwrapped: Vec<u8> = cocoon.unwrap(&contents)?;
        let data_u5: Vec<u5> = unwrapped
            .iter()
            .map(|x| bech32::u5::try_from_u8(*x).unwrap())
            .collect();
        let key_bytes = Vec::<u8>::from_base32(&data_u5)?;

        let mut wallet = Wallet::recover_from_utxo(&key_bytes.try_into().unwrap())?;
        wallet.set_state(Value(template.value), template.spending_counters)?;

        let settings = self.backend_client.node_client().settings()?;
        Ok(Controller {
            backend: self.backend_client.clone(),
            wallet,
            settings: settings.clone().into_wallet_settings(),
            block_date_generator: BlockDateGenerator::rolling(
                &settings,
                BlockDate {
                    epoch: 1,
                    slot_id: 0,
                },
                false,
            ),
        })
    }

    pub fn proposals(&self, group: &str) -> Result<Vec<FullProposalInfo>, Error> {
        self.backend_client.proposals(group).map_err(Into::into)
    }

    pub fn funds(&self) -> Result<Fund, Error> {
        self.backend_client.funds().map_err(Into::into)
    }

    pub fn statuses(&self) -> Result<HashMap<FragmentId, FragmentStatus>, Error> {
        self.inner.statuses().map_err(Into::into)
    }

    pub fn fragment_logs(&self) -> Result<HashMap<FragmentId, FragmentLog>, Error> {
        self.inner.fragment_logs().map_err(Into::into)
    }

    pub fn vote_plan_history(&self, vote_plan_id: VotePlanId) -> Result<Option<Vec<u8>>, Error> {
        self.inner
            .vote_plan_history(vote_plan_id)
            .map_err(Into::into)
    }

    pub fn votes_history(&self) -> Result<Option<Vec<AccountVotes>>, Error> {
        self.inner.votes_history().map_err(Into::into)
    }

    pub fn vote(
        &mut self,
        proposal: &FullProposalInfo,
        choice: Choice,
        password: &str,
    ) -> Result<FragmentId, Error> {
        let id = self.controller(password)?.vote(proposal, choice)?;
        let template = self.inner.wallets_mut().wallet_mut()?;
        template.pending_tx.push(id.into());
        Ok(id)
    }

    pub fn votes_batch(
        &mut self,
        votes_data: Vec<(&FullProposalInfo, Choice)>,
        password: &str,
    ) -> Result<Vec<FragmentId>, Error> {
        let ids = self.controller(password)?.votes_batch(votes_data)?;
        let template = self.inner.wallets_mut().wallet_mut()?;
        let template_ids: Vec<jormungandr_lib::crypto::hash::Hash> =
            ids.iter().cloned().map(Into::into).collect();
        template.pending_tx.extend(template_ids);
        Ok(ids)
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Bech32(#[from] bech32::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serde(#[from] serde_yaml::Error),
    #[error(transparent)]
    Config(#[from] thor::cli::ConfigError),
    #[error(transparent)]
    Inner(#[from] thor::cli::Error),
    #[error("cannot serialize secret key")]
    CannotrSerializeSecretKey(#[from] bincode::ErrorKind),
    #[error(transparent)]
    Bincode(#[from] Box<bincode::ErrorKind>),
    #[error("cannot read/write secret key")]
    Cocoon,
    #[error(transparent)]
    Valgrind(#[from] valgrind::Error),
    #[error(transparent)]
    Backend(#[from] RestError),
    #[error(transparent)]
    Controller(#[from] iapyx::ControllerError),
    #[error(transparent)]
    Wallet(#[from] iapyx::WalletError),
    #[error(transparent)]
    Read(#[from] chain_core::property::ReadError),
    #[error(transparent)]
    WalletCore(#[from] wallet_core::Error),
}

impl From<cocoon::Error> for Error {
    fn from(_err: cocoon::Error) -> Self {
        Error::Cocoon
    }
}
