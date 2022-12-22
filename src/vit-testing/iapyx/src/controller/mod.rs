mod builder;

use crate::Wallet;
pub use builder::{ControllerBuilder, Error as ControllerBuilderError};
use chain_impl_mockchain::fragment::FragmentId;
use jormungandr_automation::jormungandr::RestError;
use jormungandr_automation::jormungandr::RestSettings;
use jormungandr_lib::interfaces::AccountVotes;
use jormungandr_lib::interfaces::Address;
use jormungandr_lib::interfaces::SettingsDto;
use jormungandr_lib::interfaces::VotePlanId;
use jormungandr_lib::interfaces::{AccountState, FragmentLog, FragmentStatus};
use std::collections::HashMap;
use thiserror::Error;
use thor::BlockDateGenerator;
use thor::DiscriminationExtension;
use valgrind::ProposalExtension;
use valgrind::{Fund, ValgrindClient};
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
use wallet::{AccountId, Settings};
use wallet_core::{Choice, Value};

pub struct Controller {
    pub backend: ValgrindClient,
    pub wallet: Wallet,
    pub settings: Settings,
    pub block_date_generator: BlockDateGenerator,
}

impl Controller {
    pub fn switch_backend(
        &mut self,
        proxy_address: String,
        backend_settings: RestSettings,
    ) -> Result<(), ControllerError> {
        self.backend = ValgrindClient::new(proxy_address, backend_settings)?;
        Ok(())
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

    pub fn set_block_date_generator(&mut self, block_date_generator: BlockDateGenerator) {
        self.block_date_generator = block_date_generator;
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
                            self.remove_pending_transaction(*id);
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

    pub fn remove_pending_transaction(&mut self, id: FragmentId) {
        self.wallet.remove_pending_transaction(id);
    }

    pub fn total_value(&self) -> Value {
        self.wallet.total_value()
    }

    pub fn refresh_state(&mut self) -> Result<(), ControllerError> {
        let account_state = self.get_account_state()?;
        let value: u64 = (*account_state.value()).into();
        self.wallet
            .set_state(Value(value), account_state.counters());
        Ok(())
    }

    pub fn get_account_state(&self) -> Result<AccountState, ControllerError> {
        self.backend.account_state(self.id()).map_err(Into::into)
    }

    pub fn proposals(&self, group: &str) -> Result<Vec<FullProposalInfo>, ControllerError> {
        self.backend.proposals(group).map_err(Into::into)
    }

    pub fn funds(&self) -> Result<Fund, ControllerError> {
        self.backend.funds().map_err(Into::into)
    }

    pub fn settings(&self) -> Result<SettingsDto, ControllerError> {
        self.backend.settings().map_err(Into::into)
    }

    pub fn vote_for(
        &mut self,
        voteplan_id: String,
        proposal_index: u32,
        choice: u8,
    ) -> Result<FragmentId, ControllerError> {
        let funds = self.funds()?;

        let voteplan = funds
            .chain_vote_plans
            .iter()
            .find(|vp| voteplan_id == vp.chain_voteplan_id)
            .unwrap();

        let group = funds
            .groups
            .iter()
            .find(|g| g.token_identifier == voteplan.token_identifier)
            .unwrap();

        let proposals = self.get_proposals(&group.group_id)?;

        let proposal = proposals
            .iter()
            .find(|x| {
                x.voteplan.chain_voteplan_id == voteplan_id
                    && x.voteplan.chain_proposal_index == proposal_index as i64
            })
            .ok_or(ControllerError::CannotFindProposal {
                vote_plan_name: voteplan_id.to_string(),
                proposal_index,
            })?;

        self.vote(proposal, Choice::new(choice))
    }

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
                    .set_state((*account_state.value()).into(), counters);
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

    pub fn get_proposals(&mut self, group: &str) -> Result<Vec<FullProposalInfo>, ControllerError> {
        Ok(self.backend.proposals(group)?.to_vec())
    }

    pub fn fragment_logs(&self) -> Result<HashMap<FragmentId, FragmentLog>, ControllerError> {
        Ok(self.backend.fragment_logs()?)
    }

    pub fn vote_plan_history(
        &self,
        vote_plan_id: VotePlanId,
    ) -> Result<Option<Vec<u8>>, ControllerError> {
        self.backend
            .vote_plan_history(self.wallet_address(), vote_plan_id)
            .map_err(Into::into)
    }

    pub fn votes_history(&self) -> Result<Option<Vec<AccountVotes>>, ControllerError> {
        self.backend
            .votes_history(self.wallet_address())
            .map_err(Into::into)
    }

    fn wallet_address(&self) -> Address {
        self.wallet
            .identifier(self.settings.discrimination)
            .into_address(
                self.settings.discrimination,
                &self.settings.discrimination.into_prefix(),
            )
    }

    pub fn active_vote_plan(
        &self,
    ) -> Result<Vec<jormungandr_lib::interfaces::VotePlanStatus>, ControllerError> {
        Ok(self.backend.active_vote_plan()?)
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
    #[error(transparent)]
    Rest(#[from] RestError),
}
