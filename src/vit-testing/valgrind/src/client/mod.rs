mod node;
mod proxy;
pub mod utils;
mod vit_station;

use chain_core::packer::Codec;
use chain_core::property::Fragment as _;
use chain_impl_mockchain::fragment::{Fragment, FragmentId};
use chain_ser::deser::Deserialize;
use chain_ser::deser::ReadError;
use jormungandr_automation::jormungandr::Explorer;
pub use jormungandr_automation::jormungandr::RestSettings as ValgrindSettings;
use jormungandr_lib::interfaces::AccountVotes;
use jormungandr_lib::interfaces::Address;
use jormungandr_lib::interfaces::FragmentStatus;
use jormungandr_lib::interfaces::SettingsDto;
use jormungandr_lib::interfaces::VotePlanId;
use jormungandr_lib::interfaces::{AccountState, FragmentLog, VotePlanStatus};
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;
use vit_servicing_station_lib::db::models::challenges::Challenge;
use vit_servicing_station_lib::db::models::community_advisors_reviews::AdvisorReview;
use vit_servicing_station_lib::db::models::funds::Fund;
use vit_servicing_station_lib::db::models::proposals::Proposal;
use wallet::AccountId;

pub use node::{RestError as NodeRestError, WalletNodeRestClient};
pub use proxy::{Error as ProxyClientError, ProxyClient};
use url::Url;
use vit_servicing_station_tests::common::clients::RestClient as VitRestClient;
pub use vit_station::{RestError as VitStationRestError, VitStationRestClient};

#[derive(Clone)]
pub struct ValgrindClient {
    node_client: WalletNodeRestClient,
    vit_client: VitRestClient,
    proxy_client: ProxyClient,
    explorer_client: Explorer,
}

impl ValgrindClient {
    pub fn new_from_addresses(
        proxy_address: Url,
        node_address: Url,
        vit_address: Url,
        node_rest_settings: ValgrindSettings,
    ) -> Self {
        println!(
            "node_address: {}",
            node_address.join("api").unwrap().as_str()
        );
        println!("vit_address: {:?}", vit_address.to_string());
        println!("proxy_address: {:?}", proxy_address.to_string());

        let mut backend = Self {
            node_client: WalletNodeRestClient::new(
                node_address.join("api").unwrap(),
                node_rest_settings.clone(),
            ),
            vit_client: VitRestClient::new(vit_address),
            proxy_client: ProxyClient::new(proxy_address.to_string()),
            explorer_client: Explorer::new(node_address.to_string(), None),
        };

        if node_rest_settings.enable_debug {
            backend.enable_logs();
        }
        backend
    }

    pub fn new(address: String, settings: ValgrindSettings) -> Result<Self, Error> {
        let proxy_address: Url = address.parse()?;
        Ok(Self::new_from_addresses(
            proxy_address.clone(),
            proxy_address.clone(),
            proxy_address,
            settings,
        ))
    }

    pub fn node_client(&self) -> WalletNodeRestClient {
        self.node_client.clone()
    }

    pub fn send_fragment(&self, transaction: Vec<u8>) -> Result<FragmentId, Error> {
        self.node_client.send_fragment(transaction.clone())?;
        let fragment = Fragment::deserialize(&mut Codec::new(transaction.as_slice()))?;
        Ok(fragment.id())
    }

    pub fn send_fragments(&self, transactions: Vec<Vec<u8>>) -> Result<Vec<FragmentId>, Error> {
        for tx in transactions.iter() {
            self.node_client.send_fragment(tx.clone())?;
        }
        Ok(transactions
            .iter()
            .map(|tx| {
                Fragment::deserialize(&mut Codec::new(tx.as_slice()))
                    .unwrap()
                    .id()
            })
            .collect())
    }

    pub fn send_fragments_at_once(
        &self,
        transactions: Vec<Vec<u8>>,
        use_v1: bool,
    ) -> Result<Vec<FragmentId>, Error> {
        self.node_client
            .send_fragments(transactions.clone(), use_v1)?;
        Ok(transactions
            .iter()
            .map(|tx| {
                Fragment::deserialize(&mut Codec::new(tx.as_slice()))
                    .unwrap()
                    .id()
            })
            .collect())
    }

    pub fn fragment_logs(&self) -> Result<HashMap<FragmentId, FragmentLog>, Error> {
        self.node_client.fragment_logs().map_err(Into::into)
    }

    pub fn fragments_statuses(
        &self,
        ids: Vec<String>,
    ) -> Result<HashMap<FragmentId, FragmentStatus>, Error> {
        self.node_client.fragment_statuses(ids).map_err(Into::into)
    }

    pub fn account_state(&self, account_id: AccountId) -> Result<AccountState, Error> {
        self.node_client
            .account_state(account_id)
            .map_err(Into::into)
    }

    pub fn proposals(&self) -> Result<Vec<Proposal>, Error> {
        Ok(self
            .vit_client
            .proposals()?
            .iter()
            .cloned()
            .map(Into::into)
            .collect())
    }

    pub fn funds(&self) -> Result<Fund, Error> {
        Ok(self.vit_client.funds()?)
    }

    pub fn review(&self, proposal_id: &str) -> Result<HashMap<String, Vec<AdvisorReview>>, Error> {
        Ok(self.vit_client.advisor_reviews(proposal_id)?)
    }

    pub fn challenges(&self) -> Result<Vec<Challenge>, Error> {
        Ok(self.vit_client.challenges()?)
    }

    pub fn vit(&self) -> VitRestClient {
        self.vit_client.clone()
    }

    pub fn block0(&self) -> Result<Vec<u8>, Error> {
        Ok(self.proxy_client.block0().map(Into::into)?)
    }

    pub fn vote_plan_statuses(&self) -> Result<Vec<VotePlanStatus>, Error> {
        self.node_client.vote_plan_statuses().map_err(Into::into)
    }

    pub fn disable_logs(&mut self) {
        self.node_client.disable_logs();
        self.vit_client.disable_log();
        self.proxy_client.disable_debug();
    }

    pub fn enable_logs(&mut self) {
        self.node_client.enable_logs();
        self.proxy_client.enable_debug();
    }

    pub fn are_fragments_in_blockchain(
        &self,
        fragment_ids: Vec<FragmentId>,
    ) -> Result<bool, Error> {
        Ok(fragment_ids.iter().all(|x| {
            let hash = jormungandr_lib::crypto::hash::Hash::from_str(&x.to_string()).unwrap();
            self.explorer_client.transaction(hash).is_ok()
        }))
    }

    pub fn active_vote_plan(&self) -> Result<Vec<VotePlanStatus>, Error> {
        self.node_client.vote_plan_statuses().map_err(Into::into)
    }

    pub fn vote_plan_history(
        &self,
        address: Address,
        vote_plan_id: VotePlanId,
    ) -> Result<Option<Vec<u8>>, Error> {
        self.node_client
            .account_votes_for_plan(vote_plan_id, address)
            .map_err(Into::into)
    }

    pub fn votes_history(&self, address: Address) -> Result<Option<Vec<AccountVotes>>, Error> {
        self.node_client.account_votes(address).map_err(Into::into)
    }

    pub fn settings(&self) -> Result<SettingsDto, Error> {
        self.node_client.settings().map_err(Into::into)
    }

    pub fn account_exists(&self, id: AccountId) -> Result<bool, Error> {
        self.node_client.account_exists(id).map_err(Into::into)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("vit station error")]
    VitStationConnection(#[from] VitStationRestError),
    #[error(transparent)]
    NodeConnection(#[from] NodeRestError),
    #[error(transparent)]
    ProxyConnection(#[from] ProxyClientError),
    #[error("io error")]
    IoError(#[from] std::io::Error),
    #[error("block0 retrieve error")]
    SettingsRead(#[from] Box<chain_impl_mockchain::ledger::Error>),
    #[error("cannot convert hash")]
    HashConversion(#[from] chain_crypto::hash::Error),
    #[error(transparent)]
    VitRest(#[from] vit_servicing_station_tests::common::clients::RestError),
    #[error(transparent)]
    Url(#[from] url::ParseError),
    #[error(transparent)]
    Read(#[from] ReadError),
}
