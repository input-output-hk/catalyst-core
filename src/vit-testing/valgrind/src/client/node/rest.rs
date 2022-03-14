use chain_core::property::Deserialize;
use chain_crypto::{bech32::Bech32, Ed25519, PublicKey};
use chain_impl_mockchain::fragment::{Fragment, FragmentId};
pub use jormungandr_automation::jormungandr::{JormungandrRest, RestError, RestSettings};
use jormungandr_lib::interfaces::{
    AccountState, AccountVotes, Address, FragmentLog, FragmentStatus, NodeStatsDto, SettingsDto,
    VotePlanId, VotePlanStatus,
};
use std::collections::HashMap;
use std::str::FromStr;
use url::Url;
use wallet::AccountId;

#[derive(Clone)]
pub struct WalletNodeRestClient {
    rest_client: JormungandrRest,
}

impl WalletNodeRestClient {
    pub fn new(address: Url, settings: RestSettings) -> Self {
        Self {
            rest_client: JormungandrRest::new_with_custom_settings(address.to_string(), settings),
        }
    }

    pub fn send_fragment(&self, body: Vec<u8>) -> Result<(), RestError> {
        self.rest_client.send_raw_fragment(body)
    }

    pub fn send_fragments(&self, bodies: Vec<Vec<u8>>, use_v1: bool) -> Result<(), RestError> {
        if use_v1 {
            self.rest_client.send_fragment_batch(
                bodies
                    .iter()
                    .map(|tx| Fragment::deserialize(tx.as_slice()).unwrap())
                    .collect(),
                true,
            )?;
            Ok(())
        } else {
            self.rest_client.send_raw_fragments(bodies)
        }
    }

    pub fn fragment_logs(&self) -> Result<HashMap<FragmentId, FragmentLog>, RestError> {
        Ok(self
            .rest_client
            .fragment_logs()?
            .iter()
            .map(|(id, entry)| {
                let str = id.to_string();
                (FragmentId::from_str(&str).unwrap(), entry.clone())
            })
            .collect())
    }

    pub fn fragment_statuses(
        &self,
        statuses: Vec<String>,
    ) -> Result<HashMap<FragmentId, FragmentStatus>, RestError> {
        Ok(self
            .rest_client
            .fragments_statuses(statuses)?
            .iter()
            .map(|(id, entry)| {
                let str = id.to_string();
                (FragmentId::from_str(&str).unwrap(), entry.clone())
            })
            .collect())
    }

    pub fn disable_logs(&mut self) {
        self.rest_client.disable_logger();
    }

    pub fn enable_logs(&mut self) {
        self.rest_client.enable_logger();
    }

    pub fn stats(&self) -> Result<NodeStatsDto, RestError> {
        self.rest_client.stats()
    }

    pub fn account_state(&self, account_id: AccountId) -> Result<AccountState, RestError> {
        let public_key: PublicKey<Ed25519> = account_id.into();
        self.account_state_by_pk(public_key.to_bech32_str())
    }

    pub fn account_state_by_pk(&self, bech32: String) -> Result<AccountState, RestError> {
        self.rest_client.account_state_by_pk(&bech32)
    }

    pub fn settings(&self) -> Result<SettingsDto, RestError> {
        self.rest_client.settings()
    }

    pub fn account_exists(&self, account_id: AccountId) -> Result<bool, RestError> {
        let public_key: PublicKey<Ed25519> = account_id.into();
        let response_text = self
            .rest_client
            .account_state_by_pk_raw(&public_key.to_bech32_str())?;
        Ok(!response_text.is_empty())
    }

    pub fn vote_plan_statuses(&self) -> Result<Vec<VotePlanStatus>, RestError> {
        self.rest_client.vote_plan_statuses()
    }

    pub fn account_votes_for_plan(
        &self,
        vote_plan_id: VotePlanId,
        address: Address,
    ) -> Result<Option<Vec<u8>>, RestError> {
        self.rest_client
            .account_votes_with_plan_id(vote_plan_id, address)
    }

    pub fn account_votes(&self, address: Address) -> Result<Option<Vec<AccountVotes>>, RestError> {
        self.rest_client.account_votes(address)
    }
}
