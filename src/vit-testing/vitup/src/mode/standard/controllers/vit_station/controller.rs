#![allow(dead_code)]

use super::{RestClient, Result};
use jormungandr_automation::jormungandr::NodeAlias;
use jormungandr_automation::jormungandr::Status;
use jormungandr_automation::testing::NamedProcess;
use std::net::SocketAddr;
use std::process::Child;
use std::sync::{Arc, Mutex};
use vit_servicing_station_lib::db::models::challenges::Challenge;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
use vit_servicing_station_tests::common::raw_snapshot::RawSnapshot;
use vit_servicing_station_tests::common::snapshot::VoterInfo;

pub type VitStationSettings = vit_servicing_station_lib::server::settings::ServiceSettings;

//TODO: encapsulate fields
pub struct VitStationController {
    pub(crate) alias: NodeAlias,
    pub(crate) rest_client: RestClient,
    pub(crate) settings: VitStationSettings,
    pub(crate) status: Arc<Mutex<Status>>,
    pub(crate) process: Child,
}

pub const VIT_CONFIG: &str = "vit_config.yaml";
pub const STORAGE: &str = "storage.db";
pub const VIT_STATION_LOG: &str = "vit_station.log";

impl VitStationController {
    pub fn alias(&self) -> &NodeAlias {
        &self.alias
    }

    pub fn status(&self) -> Status {
        // FIXME: this is basically a Clone, but it has to be implemented in
        // jormungandr_automatation, this is only just for the sake of making it compile
        match *self.status.lock().unwrap() {
            Status::Running => Status::Running,
            Status::Starting => Status::Starting,
            Status::Exited(e) => Status::Exited(e),
        }
    }

    pub fn check_running(&self) -> bool {
        self.rest_client.health().is_ok()
    }

    pub fn address(&self) -> SocketAddr {
        self.settings.address
    }

    pub fn proposals(&self, group: &str) -> Result<Vec<FullProposalInfo>> {
        Ok(self.rest_client.proposals(group)?)
    }

    pub fn challenges(&self) -> Result<Vec<Challenge>> {
        Ok(self.rest_client.challenges()?)
    }

    pub fn put_raw_snapshot(&self, raw_snapshot: &RawSnapshot) -> Result<()> {
        Ok(self.rest_client.put_raw_snapshot(raw_snapshot)?)
    }

    pub fn snapshot_tags(&self) -> Result<Vec<String>> {
        Ok(self.rest_client.snapshot_tags()?)
    }

    pub fn voter_info(&self, tag: &str, key: &str) -> Result<VoterInfo> {
        Ok(self.rest_client.voter_info(tag, key)?)
    }

    pub fn as_named_process(&self) -> NamedProcess {
        NamedProcess::new(self.alias().to_string(), self.process.id() as usize)
    }

    pub fn shutdown(&mut self) {
        let _ = self.process.kill();
    }

    pub fn wait(&mut self) -> Result<Status> {
        self.process.wait().map(Status::Exited).map_err(Into::into)
    }
}

impl Drop for VitStationController {
    fn drop(&mut self) {
        self.shutdown();
        self.wait().unwrap();
    }
}
