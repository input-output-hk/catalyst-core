mod builder;

pub use builder::{ControllerBuilder, Error as ControllerBuilderError};

use crate::utils::valid_until::ValidUntil;
use crate::Wallet;
use chain_impl_mockchain::{fragment::FragmentId, transaction::Input};
use jormungandr_lib::interfaces::{AccountState, FragmentLog, FragmentStatus};
use jormungandr_testing_utils::testing::node::RestSettings;
use std::collections::HashMap;
use thiserror::Error;
use valgrind::{Proposal as ValgrindProposal, SimpleVoteStatus, ValgrindClient};
use wallet::{AccountId, Settings};
use wallet_core::{Choice, Value};

pub struct Controller {
    pub(self) backend: ValgrindClient,
    pub(self) wallet: Wallet,
    pub(self) settings: Settings,
}

impl Controller {
    pub fn switch_backend(&mut self, proxy_address: String, backend_settings: RestSettings) {
        self.backend = ValgrindClient::new(proxy_address, backend_settings);
    }

    pub fn account(&self, discrimination: chain_addr::Discrimination) -> chain_addr::Address {
        self.wallet.account(discrimination)
    }

    pub fn id(&self) -> AccountId {
        self.wallet.id()
    }

    pub fn send_fragment(&self, transaction: &[u8]) -> Result<FragmentId, ControllerError> {
        self.send_fragments(vec![transaction.to_vec()])
            .map(|v| *v.first().unwrap())
    }

    pub fn send_fragments(
        &self,
        transaction: Vec<Vec<u8>>,
    ) -> Result<Vec<FragmentId>, ControllerError> {
        self.backend.send_fragments(transaction).map_err(Into::into)
    }

    pub fn confirm_all_transactions(&mut self) {
        self.wallet.confirm_all_transactions();
    }

    pub fn confirm_transaction(&mut self, id: FragmentId) {
        self.wallet.confirm_transaction(id)
    }

    pub fn pending_transactions(&self) -> Vec<FragmentId> {
        self.wallet.pending_transactions()
    }

    pub fn wait_for_pending_transactions(
        &mut self,
        pace: std::time::Duration,
    ) -> Result<(), ControllerError> {
        let mut limit = 60;
        loop {
            let ids: Vec<FragmentId> = self.pending_transactions().to_vec();

            if limit <= 0 {
                return Err(ControllerError::TransactionsWerePendingForTooLong { fragments: ids });
            }

            if ids.is_empty() {
                return Ok(());
            }

            let fragment_logs = self.backend.fragment_logs().unwrap();
            for id in ids.iter() {
                if let Some(fragment) = fragment_logs.get(id) {
                    match fragment.status() {
                        FragmentStatus::Rejected { .. } => {
                            self.remove_pending_transaction(id);
                        }
                        FragmentStatus::InABlock { .. } => {
                            self.confirm_transaction(*id);
                        }
                        _ => (),
                    };
                }
            }

            if ids.is_empty() {
                return Ok(());
            } else {
                std::thread::sleep(pace);
                limit += 1;
            }
        }
    }

    pub fn remove_pending_transaction(&mut self, id: &FragmentId) -> Option<Vec<Input>> {
        self.wallet.remove_pending_transaction(id)
    }

    pub fn total_value(&self) -> Value {
        self.wallet.total_value()
    }

    pub fn refresh_state(&mut self) -> Result<(), ControllerError> {
        let account_state = self.get_account_state()?;
        let value: u64 = (*account_state.value()).into();
        self.wallet.set_state(Value(value), account_state.counter());
        Ok(())
    }

    pub fn get_account_state(&self) -> Result<AccountState, ControllerError> {
        self.backend.account_state(self.id()).map_err(Into::into)
    }

    pub fn proposals(&self) -> Result<Vec<ValgrindProposal>, ControllerError> {
        self.backend.proposals().map_err(Into::into)
    }

    pub fn settings(&self) -> Result<Settings, ControllerError> {
        self.backend.settings().map_err(Into::into)
    }

    pub fn vote_for(
        &mut self,
        vote_plan_id: String,
        proposal_index: u32,
        choice: u8,
        valid_until: ValidUntil,
    ) -> Result<FragmentId, ControllerError> {
        let proposals = self.get_proposals()?;

        let proposal = proposals
            .iter()
            .find(|x| {
                x.chain_voteplan_id == vote_plan_id
                    && x.chain_proposal_index == proposal_index as i64
            })
            .ok_or(ControllerError::CannotFindProposal {
                vote_plan_name: vote_plan_id.to_string(),
                proposal_index,
            })?;

        self.vote(proposal, Choice::new(choice), &valid_until)
    }

    pub fn vote(
        &mut self,
        proposal: &ValgrindProposal,
        choice: Choice,
        valid_until: &ValidUntil,
    ) -> Result<FragmentId, ControllerError> {
        let valid_until_block_date =
            valid_until.into_expiry_date(Some(self.backend.settings()?))?;

        let transaction = self.wallet.vote(
            self.settings.clone(),
            &proposal.clone().into(),
            choice,
            &valid_until_block_date,
        )?;
        Ok(self.backend.send_fragment(transaction.to_vec())?)
    }

    pub fn votes_batch(
        &mut self,
        votes_data: Vec<(&ValgrindProposal, Choice)>,
        valid_until: &ValidUntil,
    ) -> Result<Vec<FragmentId>, ControllerError> {
        let account_state = self.backend.account_state(self.wallet.id())?;
        let valid_until_block_date =
            valid_until.into_expiry_date(Some(self.backend.settings()?))?;

        let mut counter = account_state.counter();
        let settings = self.settings.clone();
        let txs = votes_data
            .into_iter()
            .map(|(p, c)| {
                self.wallet
                    .set_state((*account_state.value()).into(), counter);
                let tx = self
                    .wallet
                    .vote(
                        settings.clone(),
                        &p.clone().into(),
                        c,
                        &valid_until_block_date,
                    )
                    .unwrap()
                    .to_vec();
                counter += 1;
                tx
            })
            .rev()
            .collect();

        self.backend
            .send_fragments_at_once(txs, true)
            .map_err(Into::into)
    }

    pub fn get_proposals(&mut self) -> Result<Vec<ValgrindProposal>, ControllerError> {
        Ok(self
            .backend
            .proposals()?
            .iter()
            .cloned()
            .map(Into::into)
            .collect())
    }

    pub fn fragment_logs(&self) -> Result<HashMap<FragmentId, FragmentLog>, ControllerError> {
        Ok(self.backend.fragment_logs()?)
    }

    pub fn active_votes(&self) -> Result<Vec<SimpleVoteStatus>, ControllerError> {
        Ok(self
            .backend
            .vote_statuses(self.wallet.identifier(self.settings.discrimination))?)
    }
}

#[derive(Debug, Error)]
pub enum ControllerError {
    #[error("cannot find proposal: voteplan({vote_plan_name}) index({proposal_index})")]
    CannotFindProposal {
        vote_plan_name: String,
        proposal_index: u32,
    },
    #[error("transactions with ids [{fragments:?}] were pending for too long")]
    TransactionsWerePendingForTooLong { fragments: Vec<FragmentId> },
    #[error(transparent)]
    Valgrind(#[from] valgrind::Error),
    #[error(transparent)]
    WalletTime(#[from] wallet::time::Error),
    #[error(transparent)]
    Wallet(#[from] crate::wallet::Error),
}
