mod node;
mod proxy;
mod vit_station;

use crate::data::AdvisorReview;
use crate::data::Challenge;
use crate::Fund;
use crate::Proposal;
use crate::SimpleVoteStatus;
use chain_core::property::Fragment as _;
use chain_impl_mockchain::config::Block0Date;
use chain_impl_mockchain::fragment::{Fragment, FragmentId};
use chain_impl_mockchain::key::Hash;
use chain_ser::deser::Deserialize;
use jormungandr_lib::interfaces::AccountIdentifier;
use jormungandr_lib::interfaces::FragmentStatus;
use jormungandr_lib::interfaces::{AccountState, FragmentLog, VotePlanStatus};
use jormungandr_testing_utils::testing::node::Explorer;
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;
use wallet::{AccountId, Settings};

pub use jormungandr_testing_utils::testing::node::RestSettings as ValgrindSettings;
pub use node::{RestError as NodeRestError, WalletNodeRestClient};
pub use proxy::{Error as ProxyClientError, ProxyClient};
pub use vit_station::{RestError as VitStationRestError, VitStationRestClient};

#[derive(Clone)]
pub struct ValgrindClient {
    node_client: WalletNodeRestClient,
    vit_client: VitStationRestClient,
    proxy_client: ProxyClient,
    explorer_client: Explorer,
}

impl ValgrindClient {
    pub fn new_from_addresses(
        proxy_address: String,
        node_address: String,
        vit_address: String,
        node_rest_settings: ValgrindSettings,
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

    pub fn new(address: String, settings: ValgrindSettings) -> Self {
        Self::new_from_addresses(address.clone(), address.clone(), address, settings)
    }

    pub fn node_client(&self) -> WalletNodeRestClient {
        self.node_client.clone()
    }

    pub fn send_fragment(&self, transaction: Vec<u8>) -> Result<FragmentId, Error> {
        self.node_client.send_fragment(transaction.clone())?;
        let fragment = Fragment::deserialize(transaction.as_slice())?;
        Ok(fragment.id())
    }

    pub fn send_fragments(&self, transactions: Vec<Vec<u8>>) -> Result<Vec<FragmentId>, Error> {
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
    ) -> Result<Vec<FragmentId>, Error> {
        self.node_client
            .send_fragments(transactions.clone(), use_v1)?;
        Ok(transactions
            .iter()
            .map(|tx| Fragment::deserialize(tx.as_slice()).unwrap().id())
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
        Ok(self.vit_client.review(proposal_id)?)
    }

    pub fn challenges(&self) -> Result<Vec<Challenge>, Error> {
        Ok(self.vit_client.challenges()?)
    }

    pub fn block0(&self) -> Result<Vec<u8>, Error> {
        Ok(self.proxy_client.block0().map(Into::into)?)
    }

    pub fn vote_plan_statuses(&self) -> Result<Vec<VotePlanStatus>, Error> {
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
    ) -> Result<bool, Error> {
        Ok(fragment_ids.iter().all(|x| {
            let hash = jormungandr_lib::crypto::hash::Hash::from_str(&x.to_string()).unwrap();
            self.explorer_client.transaction(hash).is_ok()
        }))
    }

    pub fn vote_statuses(
        &self,
        _identifier: AccountIdentifier,
    ) -> Result<Vec<SimpleVoteStatus>, Error> {
        unimplemented!()
    }

    pub fn settings(&self) -> Result<Settings, Error> {
        use chain_time::SlotDuration;
        use chain_time::TimeFrame;
        use chain_time::Timeline;
        use chain_time::{Epoch, TimeEra};

        let settings = self.node_client.settings()?;
        let duration_since_epoch = settings.block0_time.duration_since_epoch();
        let timeline = Timeline::new(std::time::SystemTime::now());
        let tf = TimeFrame::new(
            timeline,
            SlotDuration::from_secs(settings.slot_duration as u32),
        );
        let slot0 = tf.slot0();
        let era = TimeEra::new(slot0, Epoch(0), settings.slots_per_epoch);

        Ok(Settings {
            fees: settings.fees,
            discrimination: settings.discrimination,
            block0_initial_hash: Hash::from_str(&settings.block0_hash)?,
            block0_date: Block0Date(duration_since_epoch.as_secs()),
            slot_duration: settings.slot_duration as u8,
            time_era: era,
            transaction_max_expiry_epochs: settings.tx_max_expiry_epochs,
        })
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
    Block0Read(#[from] chain_core::mempack::ReadError),
    #[error("block0 retrieve error")]
    SettingsRead(#[from] Box<chain_impl_mockchain::ledger::Error>),
    #[error("cannot convert hash")]
    HashConversion(#[from] chain_crypto::hash::Error),
}
