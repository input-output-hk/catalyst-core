use chain_impl_mockchain::fragment::FragmentId;
use jormungandr_lib::interfaces::FragmentStatus;
use jortestkit::load::RequestStatusProvider;
use jortestkit::load::{Id, Status};
use thiserror::Error;
use valgrind::ValgrindClient;

/// Responsible for providing information about status for given ids collection.
/// It can work over a batch of ids by preparing request to Jormungandr V1 REST Api. Basically it query
/// statues for given ids and prepare `Status` struct for each id
pub struct VoteStatusProvider {
    backend: ValgrindClient,
}

impl VoteStatusProvider {
    /// Creates object based on address and verbosity
    pub fn new(backend_address: String, debug: bool) -> Result<Self, Error> {
        let mut backend = ValgrindClient::new(backend_address, Default::default())?;
        if debug {
            backend.enable_logs();
        } else {
            backend.disable_logs();
        }
        Ok(Self { backend })
    }
}

impl RequestStatusProvider for VoteStatusProvider {
    fn get_statuses(&self, ids: &[Id]) -> Vec<Status> {
        match self
            .backend
            .fragments_statuses(ids.iter().map(|id| id.to_string()).collect())
        {
            Ok(fragments_statuses) => fragments_statuses
                .iter()
                .map(|(id, fragment_log)| into_status(fragment_log, id))
                .collect(),
            Err(_) => vec![],
        }
    }
}

fn into_status(fragment_status: &FragmentStatus, id: &FragmentId) -> Status {
    match fragment_status {
        FragmentStatus::Pending => {
            Status::new_pending(std::time::Duration::from_secs(0), id.to_string())
        }
        FragmentStatus::Rejected { reason } => Status::new_failure(
            std::time::Duration::from_secs(0),
            id.to_string(),
            reason.to_string(),
        ),
        FragmentStatus::InABlock { .. } => {
            Status::new_success(std::time::Duration::from_secs(0), id.to_string())
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("wallet error")]
    Wallet(#[from] crate::wallet::Error),
    #[error("wallet error")]
    Backend(#[from] valgrind::Error),
    #[error("controller error")]
    Controller(#[from] crate::ControllerError),
    #[error("pin read error")]
    PinRead(#[from] crate::utils::qr::Error),
    #[error("wallet time error")]
    WalletTime(#[from] wallet::time::Error),
}
