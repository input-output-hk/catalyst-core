mod node;
mod proxy;
mod vit_station;

use crate::data::AdvisorReview;
use crate::data::Challenge;
use crate::Fund;
use crate::Proposal;
use crate::SimpleVoteStatus;
use chain_core::property::Fragment as _;
use chain_impl_mockchain::fragment::{Fragment, FragmentId};
use chain_impl_mockchain::key::Hash;
use chain_ser::deser::Deserialize;
use jormungandr_lib::interfaces::AccountIdentifier;
use jormungandr_lib::interfaces::FragmentStatus;
use jormungandr_lib::interfaces::{AccountState, FragmentLog, VotePlanStatus};
use jormungandr_testing_utils::testing::node::Explorer;
pub use jormungandr_testing_utils::testing::node::RestSettings as WalletBackendSettings;
pub use node::{RestError as NodeRestError, WalletNodeRestClient};
pub use proxy::{Protocol, ProxyClient, ProxyClientError, ProxyServerError, ProxyServerStub};
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;
pub use vit_station::{RestError as VitRestError, VitStationRestClient};
use wallet::{AccountId, Settings};

#[derive(Clone)]
pub struct WalletBackend {
    node_client: WalletNodeRestClient,
    vit_client: VitStationRestClient,
    proxy_client: ProxyClient,
    explorer_client: Explorer,
}

impl WalletBackend {
    pub fn new_from_addresses(
        proxy_address: String,
        node_address: String,
        vit_address: String,
        node_rest_settings: WalletBackendSettings,
    ) -> Self {
        let mut backend = Self {
            node_client: WalletNodeRestClient::new(
                format!("http://{}/api", node_address),
                node_rest_settings.clone(),
            ),
            vit_client: VitStationRestClient::new(vit_address),
            proxy_client: ProxyClient::new(format!("http://{}", proxy_address)),
            explorer_client: Explorer::new(node_address),
        };

        if node_rest_settings.enable_debug {
            backend.enable_logs();
        }
        backend
    }

    pub fn new(address: String, node_rest_settings: WalletBackendSettings) -> Self {
        Self::new_from_addresses(
            address.clone(),
            address.clone(),
            address,
            node_rest_settings,
        )
    }

    pub fn node_client(&self) -> WalletNodeRestClient {
        self.node_client.clone()
    }

    pub fn send_fragment(&self, transaction: Vec<u8>) -> Result<FragmentId, WalletBackendError> {
        self.node_client.send_fragment(transaction.clone())?;
        let fragment = Fragment::deserialize(transaction.as_slice())?;
        Ok(fragment.id())
    }

    pub fn send_fragments(
        &self,
        transactions: Vec<Vec<u8>>,
    ) -> Result<Vec<FragmentId>, WalletBackendError> {
        for tx in transactions.iter() {
            self.node_client.send_fragment(tx.clone())?;
        }
        Ok(transactions
            .iter()
            .map(|tx| Fragment::deserialize(tx.as_slice()).unwrap().id())
            .collect())
    }

    pub fn send_fragments_at_once(
        &self,
        transactions: Vec<Vec<u8>>,
        use_v1: bool,
    ) -> Result<Vec<FragmentId>, WalletBackendError> {
        self.node_client
            .send_fragments(transactions.clone(), use_v1)?;
        Ok(transactions
            .iter()
            .map(|tx| Fragment::deserialize(tx.as_slice()).unwrap().id())
            .collect())
    }

    pub fn fragment_logs(&self) -> Result<HashMap<FragmentId, FragmentLog>, WalletBackendError> {
        self.node_client.fragment_logs().map_err(Into::into)
    }

    pub fn fragments_statuses(
        &self,
        ids: Vec<String>,
    ) -> Result<HashMap<FragmentId, FragmentStatus>, WalletBackendError> {
        self.node_client.fragment_statuses(ids).map_err(Into::into)
    }

    pub fn account_state(&self, account_id: AccountId) -> Result<AccountState, WalletBackendError> {
        self.node_client
            .account_state(account_id)
            .map_err(Into::into)
    }

    pub fn proposals(&self) -> Result<Vec<Proposal>, WalletBackendError> {
        Ok(self
            .vit_client
            .proposals()?
            .iter()
            .cloned()
            .map(Into::into)
            .collect())
    }

    pub fn funds(&self) -> Result<Fund, WalletBackendError> {
        Ok(self.vit_client.funds()?)
    }

    pub fn reviews(&self) -> Result<Vec<AdvisorReview>, WalletBackendError> {
        Ok(self.vit_client.reviews()?)
    }

    pub fn challenges(&self) -> Result<Vec<Challenge>, WalletBackendError> {
        Ok(self.vit_client.challenges()?)
    }

    pub fn block0(&self) -> Result<Vec<u8>, WalletBackendError> {
        Ok(self.proxy_client.block0().map(Into::into)?)
    }

    pub fn vote_plan_statuses(&self) -> Result<Vec<VotePlanStatus>, WalletBackendError> {
        self.node_client.vote_plan_statuses().map_err(Into::into)
    }

    pub fn disable_logs(&mut self) {
        self.node_client.disable_logs();
        self.vit_client.disable_logs();
        self.proxy_client.disable_debug();
    }

    pub fn enable_logs(&mut self) {
        self.node_client.enable_logs();
        self.vit_client.enable_logs();
        self.proxy_client.enable_debug();
    }

    pub fn are_fragments_in_blockchain(
        &self,
        fragment_ids: Vec<FragmentId>,
    ) -> Result<bool, WalletBackendError> {
        Ok(fragment_ids.iter().all(|x| {
            let hash = jormungandr_lib::crypto::hash::Hash::from_str(&x.to_string()).unwrap();
            self.explorer_client.transaction(hash).is_ok()
        }))
    }

    pub fn vote_statuses(
        &self,
        _identifier: AccountIdentifier,
    ) -> Result<Vec<SimpleVoteStatus>, WalletBackendError> {
        unimplemented!()
    }

    pub fn settings(&self) -> Result<Settings, WalletBackendError> {
        let settings = self.node_client.settings()?;
        Ok(Settings {
            fees: settings.fees,
            discrimination: settings.discrimination,
            block0_initial_hash: Hash::from_str(&settings.block0_hash).unwrap(),
        })
    }

    pub fn account_exists(&self, id: AccountId) -> Result<bool, WalletBackendError> {
        self.node_client.account_exists(id).map_err(Into::into)
    }
}

#[derive(Debug, Error)]
pub enum WalletBackendError {
    #[error("vit station error")]
    VitStationConnectionError(#[from] VitRestError),
    #[error("node rest error")]
    NodeConnectionError(#[from] NodeRestError),
    #[error("node rest error")]
    ProxyConnectionError(#[from] ProxyClientError),
    #[error("io error")]
    IoError(#[from] std::io::Error),
    #[error("block0 retrieve error")]
    Block0ReadError(#[from] chain_core::mempack::ReadError),
    #[error("block0 retrieve error")]
    SettingsReadError(#[from] Box<chain_impl_mockchain::ledger::Error>),
}
