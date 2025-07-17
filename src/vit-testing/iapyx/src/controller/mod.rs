mod builder;

use crate::Wallet;
pub use builder::{ControllerBuilder, Error as ControllerBuilderError};
use chain_impl_mockchain::fragment::FragmentId;
use jormungandr_automation::jormungandr::RestError;
use jormungandr_automation::jormungandr::RestSettings;
use jormungandr_lib::interfaces::Address;
use jormungandr_lib::interfaces::SettingsDto;
use jormungandr_lib::interfaces::VotePlanId;
use jormungandr_lib::interfaces::{AccountState, FragmentLog, FragmentStatus};
use jormungandr_lib::interfaces::{AccountVotes, ParseAccountIdentifierError};
use std::collections::HashMap;
use thiserror::Error;
use thor::BlockDateGenerator;
use thor::DiscriminationExtension;
use valgrind::ProposalExtension;
use valgrind::{Fund, ValgrindClient};
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
use wallet::{AccountId, Settings};
use wallet_core::{Choice, Value};

/// Responsible for all wallet operations from voting to retrieving information about proposals or
/// voting power
pub struct Controller {
    /// Catalyst backend client
    pub backend: ValgrindClient,
    /// Wallet state
    pub wallet: Wallet,
    /// Cached blockchain settings
    pub settings: Settings,
    /// Expiry date generator
    pub block_date_generator: BlockDateGenerator,
}

impl Controller {
    /// Connects to new backend
    ///
    /// # Errors
    ///
    /// On connectivity issues
    ///
    pub fn switch_backend(
        &mut self,
        proxy_address: String,
        backend_settings: RestSettings,
    ) -> Result<(), ControllerError> {
        self.backend = ValgrindClient::new(proxy_address, backend_settings)?;
        Ok(())
    }

    /// Gets account
    #[must_use]
    pub fn account(&self, discrimination: chain_addr::Discrimination) -> chain_addr::Address {
        self.wallet.account(discrimination)
    }

    /// Gets account identifier. This is something different that account method since we are not
    /// retrieving entire object but only the identifier which helps of find records in blockchain
    #[must_use]
    pub fn id(&self) -> AccountId {
        self.wallet.id()
    }

    /// Sends raw transaction bytes
    ///
    /// # Errors
    ///
    /// On connection issues
    ///
    /// # Panics
    ///
    /// On internal error in which vec does not hold transaction
    ///
    pub fn send_fragment(&self, transaction: &[u8]) -> Result<FragmentId, ControllerError> {
        self.send_fragments(vec![transaction.to_vec()])
            .map(|v| *v.first().unwrap())
    }

    /// Sets transaction ttl definition. Usually when we are sending some fragments we need to define how long
    /// we want to wait until it will be put in block. `BlockDateGenerator` helps us define ttl without calculating
    /// it each time
    pub fn set_block_date_generator(&mut self, block_date_generator: BlockDateGenerator) {
        self.block_date_generator = block_date_generator;
    }

    /// Sends collection of raw transaction bytes
    ///
    /// # Errors
    ///
    /// On connection issues
    pub fn send_fragments(
        &self,
        transaction: Vec<Vec<u8>>,
    ) -> Result<Vec<FragmentId>, ControllerError> {
        self.backend.send_fragments(transaction).map_err(Into::into)
    }

    /// Confirms all transactions means to remove them from pending transaction collection and as a result
    /// remove trace needed to track their status in node
    pub fn confirm_all_transactions(&mut self) {
        self.wallet.confirm_all_transactions();
    }

    /// Confirms transaction by it id. This means to remove it from pending transaction collection and as a result
    /// remove trace needed to track their status in node
    pub fn confirm_transaction(&mut self, id: FragmentId) {
        self.wallet.confirm_transaction(id);
    }

    /// Unconfirmed collection of transactions (which statuses we still want to track)
    #[must_use]
    pub fn pending_transactions(&self) -> Vec<FragmentId> {
        self.wallet.pending_transactions()
    }

    /// Waits until all transaction will have final states (either `InABlock` or `Reject`)
    /// for given duration
    ///
    /// # Errors
    ///
    /// On connection issues
    pub fn wait_for_pending_transactions(
        &mut self,
        pace: std::time::Duration,
    ) -> Result<(), ControllerError> {
        let mut limit = 60;
        loop {
            let ids: Vec<FragmentId> = self.pending_transactions().clone();

            if limit <= 0 {
                return Err(ControllerError::TransactionsWerePendingForTooLong { fragments: ids });
            }

            if ids.is_empty() {
                return Ok(());
            }

            let fragment_logs = self.backend.fragment_logs()?;
            for id in &ids {
                if let Some(fragment) = fragment_logs.get(id) {
                    match fragment.status() {
                        FragmentStatus::Rejected { .. } => {
                            self.remove_pending_transaction(*id);
                        }
                        FragmentStatus::InABlock { .. } => {
                            self.confirm_transaction(*id);
                        }
                        FragmentStatus::Pending => (),
                    }
                }
            }

            if ids.is_empty() {
                return Ok(());
            }

            std::thread::sleep(pace);
            limit += 1;
        }
    }

    /// remove specific transaction from assumed pending
    pub fn remove_pending_transaction(&mut self, id: FragmentId) {
        self.wallet.remove_pending_transaction(id);
    }

    /// gets total value ada
    #[must_use]
    pub fn total_value(&self) -> Value {
        self.wallet.total_value()
    }

    /// Reload wallet state from blockchain
    ///
    /// # Errors
    ///
    /// On connection issues
    pub fn refresh_state(&mut self) -> Result<(), ControllerError> {
        let account_state = self.get_account_state()?;
        let value: u64 = (*account_state.value()).into();
        self.wallet
            .set_state(Value(value), account_state.counters())
            .map_err(Into::into)
    }

    /// Get account state from blockchain (ada/delegation status etc.)
    ///
    /// # Errors
    ///
    /// On connection issues
    pub fn get_account_state(&self) -> Result<AccountState, ControllerError> {
        self.backend.account_state(self.id()).map_err(Into::into)
    }

    /// Gets proposals from vit-servicing-station
    ///
    /// # Errors
    ///
    /// On connection issues
    pub fn proposals(&self, group: &str) -> Result<Vec<FullProposalInfo>, ControllerError> {
        self.backend.proposals(group).map_err(Into::into)
    }

    /// Gets proposals from vit-servicing-station
    ///
    /// # Errors
    ///
    /// On connection issues
    pub fn funds(&self) -> Result<Fund, ControllerError> {
        self.backend.funds().map_err(Into::into)
    }

    /// Gets blockchain Settings from blockchain
    ///
    /// # Errors
    ///
    /// On connection issues
    pub fn settings(&self) -> Result<SettingsDto, ControllerError> {
        self.backend.settings().map_err(Into::into)
    }

    /// Send specialized transaction (with vote certificates) based on low level parameters
    ///
    /// # Errors
    ///
    /// On connection issues
    ///
    pub fn vote_for(
        &mut self,
        voteplan_id: &str,
        proposal_index: u32,
        choice: u8,
    ) -> Result<FragmentId, ControllerError> {
        let funds = self.funds()?;

        let voteplan = funds
            .chain_vote_plans
            .iter()
            .find(|vp| voteplan_id == vp.chain_voteplan_id)
            .ok_or_else(|| ControllerError::MissingVoteplan(voteplan_id.to_owned()))?;

        let group = funds
            .groups
            .iter()
            .find(|g| g.token_identifier == voteplan.token_identifier)
            .ok_or_else(|| {
                ControllerError::MissingGroupForToken(voteplan.token_identifier.clone())
            })?;

        let proposals = self.get_proposals(&group.group_id)?;

        let proposal = proposals
            .iter()
            .find(|x| {
                x.voteplan.chain_voteplan_id == voteplan_id
                    && x.voteplan.chain_proposal_index == i64::from(proposal_index)
            })
            .ok_or(ControllerError::CannotFindProposal {
                vote_plan_name: voteplan_id.to_string(),
                proposal_index,
            })?;

        self.vote(proposal, Choice::new(choice))
    }

    /// Send specialized transaction (with vote certificates) based on `FullProposalInfo` struct
    ///
    /// # Errors
    ///
    /// On connection issues
    ///
    pub fn vote(
        &mut self,
        proposal: &FullProposalInfo,
        choice: Choice,
    ) -> Result<FragmentId, ControllerError> {
        let transaction = self.wallet.vote(
            self.settings.clone(),
            &proposal.clone().into_wallet_proposal(),
            choice,
            &self.block_date_generator.block_date(),
        )?;
        Ok(self.backend.send_fragment(transaction.to_vec())?)
    }

    /// Send collection for specialized transaction (with vote certificates)
    /// based on `FullProposalInfo` struct
    ///
    /// # Errors
    ///
    /// On connection issues
    ///
    /// # Panics
    ///
    /// On internal error when updating wallet
    pub fn votes_batch(
        &mut self,
        votes_data: Vec<(&FullProposalInfo, Choice)>,
    ) -> Result<Vec<FragmentId>, ControllerError> {
        let account_state = self.backend.account_state(self.wallet.id())?;
        let mut counters = account_state.counters();
        let settings = self.settings.clone();
        let txs = votes_data
            .into_iter()
            .map(|(p, c)| {
                self.wallet
                    .set_state((*account_state.value()).into(), counters)
                    .unwrap();
                let tx = self
                    .wallet
                    .vote(
                        settings.clone(),
                        &p.clone().into_wallet_proposal(),
                        c,
                        &self.block_date_generator.block_date(),
                    )
                    .unwrap()
                    .to_vec();
                counters[0] += 1;
                tx
            })
            .rev()
            .collect();

        self.backend
            .send_fragments_at_once(txs, true)
            .map_err(Into::into)
    }

    /// Gets proposals from vit-servicing-station based on voting group
    ///
    /// # Errors
    ///
    /// On connection issues
    pub fn get_proposals(&mut self, group: &str) -> Result<Vec<FullProposalInfo>, ControllerError> {
        Ok(self.backend.proposals(group)?)
    }

    /// Gets fragment logs from node
    ///
    /// # Errors
    ///
    /// On connection issues
    pub fn fragment_logs(&self) -> Result<HashMap<FragmentId, FragmentLog>, ControllerError> {
        Ok(self.backend.fragment_logs()?)
    }

    /// Gets vote plan history from node for given vote plan id
    ///
    /// # Errors
    ///
    /// On connection issues
    pub fn vote_plan_history(
        &self,
        vote_plan_id: VotePlanId,
    ) -> Result<Option<Vec<u8>>, ControllerError> {
        self.backend
            .vote_plan_history(self.wallet_address()?, vote_plan_id)
            .map_err(Into::into)
    }
    /// Gets vote plan history from node
    ///
    /// # Errors
    ///
    /// On connection issues
    pub fn votes_history(&self) -> Result<Option<Vec<AccountVotes>>, ControllerError> {
        self.backend
            .votes_history(self.wallet_address()?)
            .map_err(Into::into)
    }

    fn wallet_address(&self) -> Result<Address, ControllerError> {
        Ok(self
            .wallet
            .identifier(self.settings.discrimination)?
            .into_address(
                self.settings.discrimination,
                &self.settings.discrimination.into_prefix(),
            ))
    }

    /// Gets vote plan from node
    ///
    /// # Errors
    ///
    /// On connection issues
    pub fn active_vote_plan(
        &self,
    ) -> Result<Vec<jormungandr_lib::interfaces::VotePlanStatus>, ControllerError> {
        Ok(self.backend.active_vote_plan()?)
    }
}

/// Controller related Errors
#[derive(Debug, Error)]
pub enum ControllerError {
    /// Missing proposals
    #[error("cannot find proposal: voteplan({vote_plan_name}) index({proposal_index})")]
    CannotFindProposal {
        /// name of parent vote plan
        vote_plan_name: String,
        /// proposal index within parent vote plan
        proposal_index: u32,
    },
    /// Expiry
    #[error("transactions with ids [{fragments:?}] were pending for too long")]
    TransactionsWerePendingForTooLong {
        /// Expired fragmnents
        fragments: Vec<FragmentId>,
    },
    #[error(transparent)]
    /// Proxy
    Valgrind(#[from] valgrind::Error),
    #[error(transparent)]
    /// Wallet expiry issues
    WalletTime(#[from] wallet::time::Error),
    #[error(transparent)]
    /// Wallet issues
    Wallet(#[from] crate::wallet::Error),
    #[error(transparent)]
    /// Connection with rest
    Rest(#[from] RestError),
    /// Parse account identifier
    #[error(transparent)]
    ParseAccountIdentifier(#[from] ParseAccountIdentifierError),
    ///Internal wallet errors
    #[error(transparent)]
    WalletCore(#[from] wallet_core::Error),
    ///Internal wallet errors
    #[error("cannot find voteplan with id: {0}")]
    MissingVoteplan(String),
    ///Internal wallet errors
    #[error("missing group for token identifier: {0}")]
    MissingGroupForToken(String),
}
