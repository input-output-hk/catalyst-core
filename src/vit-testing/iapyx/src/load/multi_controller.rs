use crate::utils::qr::read_qrs;
use crate::utils::qr::Error as PinReadError;
use crate::utils::qr::PinReadModeSettings;
use crate::Wallet;
use bech32::FromBase32;
use chain_impl_mockchain::{block::BlockDate, fragment::FragmentId};
use jcli_lib::key::read_bech32;
pub use jormungandr_automation::jormungandr::RestSettings;
use std::path::Path;
use thiserror::Error;
use valgrind::ProposalExtension;
use valgrind::SettingsExtensions;
use valgrind::ValgrindClient;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
use wallet::Settings;
use wallet_core::Choice;

unsafe impl Send for Wallet {}
use jormungandr_automation::testing::vit::VoteCastCounterError;
use jormungandr_lib::interfaces::VotePlanId;
use std::convert::TryInto;

/// Responsible for controlling more than one wallet at one time. Useful for load scenario or wallets
/// which handles many users
pub struct MultiController {
    pub(super) backend: ValgrindClient,
    pub(super) wallets: Vec<Wallet>,
    pub(super) settings: Settings,
}

impl MultiController {
    /// Creates object based on qr codes files
    ///
    /// # Errors
    ///
    /// On backend connectivity issues or parsing qr problems
    ///
    /// # Panics
    ///
    /// On internal error when exposing secret key
    pub fn recover_from_qrs<P: AsRef<Path>>(
        wallet_backend_address: &str,
        qrs: &[P],
        pin_mode: &PinReadModeSettings,
        backend_settings: RestSettings,
    ) -> Result<Self, MultiControllerError> {
        let mut backend =
            ValgrindClient::new(wallet_backend_address.to_string(), backend_settings)?;
        let settings = backend.settings()?.into_wallet_settings();

        backend.enable_logs();
        let wallets = read_qrs(qrs, pin_mode, true)
            .into_iter()
            .map(|secret| Wallet::recover(secret.leak_secret().as_ref()).unwrap())
            .collect();

        Ok(Self {
            backend,
            wallets,
            settings,
        })
    }

    /// Creates object based on secret files
    ///
    /// # Errors
    ///
    /// On backend connectivity issues or parsing qr problems
    ///
    /// # Panics
    ///
    /// On single wallet recover error
    ///
    pub fn recover_from_sks<P: AsRef<Path>>(
        proxy_address: &str,
        private_keys: &[P],
        backend_settings: RestSettings,
    ) -> Result<Self, MultiControllerError> {
        let backend = ValgrindClient::new(proxy_address.to_string(), backend_settings)?;
        let settings = backend.settings()?.into_wallet_settings();
        let wallets = private_keys
            .iter()
            .map(|x| {
                let (_, data, _) = read_bech32(Some(&x.as_ref().to_path_buf())).unwrap();
                let key_bytes = Vec::<u8>::from_base32(data.as_slice()).unwrap();
                let data: [u8; 64] = key_bytes.try_into().unwrap();
                Wallet::recover(&data).unwrap()
            })
            .collect();

        Ok(Self {
            backend,
            wallets,
            settings,
        })
    }

    /// Gets proposals from vit-servicing-station based on voting group
    ///
    /// # Errors
    ///
    /// On connection issues
    ///
    pub fn proposals(&self, group: &str) -> Result<Vec<FullProposalInfo>, MultiControllerError> {
        self.backend.proposals(group).map_err(Into::into)
    }

    /// Get inner backend client which can perform some custom REST API operations over the node
    /// or servicing-station
    pub(crate) fn backend(&self) -> &ValgrindClient {
        &self.backend
    }

    /// Update wallet state for entire wallets collection based on current state in blockchain
    ///
    /// # Errors
    ///
    /// On connection issues
    ///
    pub fn update_wallets_state(&mut self) -> Result<(), MultiControllerError> {
        let backend = self.backend().clone();
        let count = self.wallets.len();
        for (idx, wallet) in self.wallets.iter_mut().enumerate() {
            let account_state = backend.account_state(wallet.id())?;
            println!("{}/{} Updating account state", idx + 1, count);
            wallet.set_state((*account_state.value()).into(), account_state.counters())?;
        }
        Ok(())
    }

    /// Update wallet states based on current state in blockchain
    ///
    /// # Errors
    ///
    /// On connection issues
    ///
    pub fn update_wallet_state(&mut self, wallet_index: usize) -> Result<(), MultiControllerError> {
        let backend = self.backend().clone();
        let wallet = self
            .wallets
            .get_mut(wallet_index)
            .ok_or(MultiControllerError::NotEnoughWallets)?;
        let account_state = backend.account_state(wallet.id())?;
        wallet
            .set_state((*account_state.value()).into(), account_state.counters())
            .map_err(Into::into)
    }

    /// Update wallet states based on current state in blockchain if condition is satisfied. For example
    /// only if wallet spending counter is 0 which means it is in initial state
    ///
    /// # Errors
    ///
    /// On connection issues
    ///
    pub fn update_wallet_state_if(
        &mut self,
        wallet_index: usize,
        predicate: &dyn Fn(&Wallet) -> bool,
    ) -> Result<(), MultiControllerError> {
        let wallet = self
            .wallets
            .get_mut(wallet_index)
            .ok_or(MultiControllerError::NotEnoughWallets)?;
        if predicate(wallet) {
            self.update_wallet_state(wallet_index)?;
        }
        Ok(())
    }

    /// Sends vote transaction on behalf of wallet with index `wallet_index` on proposal with
    /// given choice. Sets expiry slot equal to `valid_until`.
    /// # Errors
    ///
    /// On connection issues
    ///
    pub fn vote(
        &mut self,
        wallet_index: usize,
        proposal: &FullProposalInfo,
        choice: Choice,
        valid_until: BlockDate,
    ) -> Result<FragmentId, MultiControllerError> {
        let wallet = self
            .wallets
            .get_mut(wallet_index)
            .ok_or(MultiControllerError::NotEnoughWallets)?;
        let tx = wallet.vote(
            self.settings.clone(),
            &proposal.clone().into_wallet_proposal(),
            choice,
            &valid_until,
        )?;
        self.backend()
            .send_fragment(tx.to_vec())
            .map_err(Into::into)
    }

    /// Sends bunch of vote transactions on behalf of wallet with index `wallet_index`
    /// with map of proposals and respectful choices. Sets expiry slot equal to `valid_until`.
    /// Method can use V0 or V1 api based on preference. V1 enable to send all transactions in a batch
    /// as single call whether legacy V0 is able to send them as one vote per call
    ///
    /// # Errors
    ///
    /// On connection issues
    ///
    /// # Panics
    ///
    /// On connection problem
    ///
    pub fn votes_batch(
        &mut self,
        wallet_index: usize,
        use_v1: bool,
        votes_data: Vec<(&FullProposalInfo, Choice)>,
        valid_until: &BlockDate,
    ) -> Result<Vec<FragmentId>, MultiControllerError> {
        let wallet = self
            .wallets
            .get_mut(wallet_index)
            .ok_or(MultiControllerError::NotEnoughWallets)?;
        let account_state = self.backend.account_state(wallet.id())?;

        let mut counters = account_state.counters();
        let settings = self.settings.clone();
        let txs = votes_data
            .into_iter()
            .map(|(p, c)| {
                wallet
                    .set_state((*account_state.value()).into(), counters)
                    .unwrap();
                let tx = wallet
                    .vote(
                        settings.clone(),
                        &p.clone().into_wallet_proposal(),
                        c,
                        valid_until,
                    )
                    .unwrap()
                    .to_vec();
                counters[0] += 1;
                tx
            })
            .rev()
            .collect();

        self.backend()
            .send_fragments_at_once(txs, use_v1)
            .map_err(Into::into)
    }

    /// Confirms all transactions for all wallets. Confirms means loose interest in tracking their statuses.
    /// This method should be call when we are sure our transactions all in final states (in block or failed).
    pub fn confirm_all_transactions(&mut self) {
        for wallet in &mut self.wallets {
            wallet.confirm_all_transactions();
        }
    }

    /// Confirms all transactions for wallet. Confirms means loose interest in tracking their statuses.
    /// This method should be call when we are sure our transactions all in final states (in block or failed).
    pub fn confirm_transaction(&mut self, fragment_id: FragmentId) {
        for wallet in &mut self.wallets {
            wallet.confirm_transaction(fragment_id);
        }
    }

    /// Wallet counts
    #[must_use]
    pub fn wallet_count(&self) -> usize {
        self.wallets.len()
    }
}

impl From<MultiController> for Vec<Wallet> {
    fn from(controller: MultiController) -> Self {
        controller.wallets
    }
}

/// Errors for `MultiController`
#[derive(Debug, Error)]
pub enum MultiControllerError {
    /// Wallet related errors
    #[error("wallet error")]
    Wallet(#[from] crate::wallet::Error),
    /// Backend related errors
    #[error("wallet error")]
    Backend(#[from] valgrind::Error),
    /// Wallet Controller related errors
    #[error("controller error")]
    Controller(#[from] crate::ControllerError),
    /// Read pin errors
    #[error("pin read error")]
    PinRead(#[from] PinReadError),
    /// Wallet time boundaries errors
    #[error("wallet time error")]
    WalletTime(#[from] wallet::time::Error),
    /// Not enough proposals
    #[error("not enough proposals")]
    NotEnoughProposals,
    /// Internal wallet core errors
    #[error(transparent)]
    WalletCore(#[from] wallet_core::Error),
    /// Not enough proposals
    #[error("not enough wallets")]
    NotEnoughWallets,
    /// Not enough votes
    #[error("not enough votes to cast")]
    NoMoreVotesToVote,
    /// Randomizing choices failed
    #[error("cannot choose next random choice")]
    RandomChoiceFailed,
    /// Missing proposal
    #[error("missing proposal with id: {0}")]
    MissingProposal(usize),
    /// Missing proposal
    #[error(transparent)]
    VotesCastRegister(#[from] VoteCastCounterError),
    /// Too many proposals
    #[error("invalid proposals length for voteplan with id: ({0}) by design it should be more than 128 proposals in single vote plan")]
    InvalidProposalsLen(VotePlanId),
}
