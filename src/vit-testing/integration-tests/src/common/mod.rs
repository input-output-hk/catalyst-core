mod assert;
pub mod load;
pub mod mainnet_wallet_ext;
pub mod registration;
mod reps;
mod rewards;
pub mod snapshot;
pub(crate) mod snapshot_filter;
mod static_data;
mod vote_plan_status;
mod wallet;

pub use reps::{empty_assigner, RepsVoterAssignerSource};

pub use assert::*;
pub use mainnet_lib::MainnetWallet;
pub use rewards::{funded_proposals, VotesRegistry};
pub use snapshot_filter::SnapshotFilter;
pub use static_data::SnapshotExtensions;
use thiserror::Error;
pub use vote_plan_status::{CastedVote, VotePlanStatusProvider};
pub use wallet::{iapyx_from_mainnet, iapyx_from_qr, iapyx_from_secret_key};

use lazy_static::lazy_static;
use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags};
use std::{
    collections::HashSet,
    sync::atomic::{AtomicU16, Ordering},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("vitup error")]
    VitupError(#[from] vitup::error::Error),
    #[error("verification error")]
    VerificationError(#[from] jormungandr_automation::testing::VerificationError),
    #[error("sender error")]
    FragmentSenderError(#[from] thor::FragmentSenderError),
    #[error("iapyx error")]
    IapyxError(#[from] iapyx::ControllerError),
}

lazy_static! {
    static ref NEXT_AVAILABLE_PORT_NUMBER: AtomicU16 = AtomicU16::new(11000);
    static ref OCCUPIED_PORTS: HashSet<u16> = {
        let af_flags = AddressFamilyFlags::IPV4;
        let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;
        get_sockets_info(af_flags, proto_flags)
            .unwrap()
            .into_iter()
            .map(|s| s.local_port())
            .collect::<HashSet<_>>()
    };
}

pub fn get_available_port() -> u16 {
    loop {
        let candidate_port = NEXT_AVAILABLE_PORT_NUMBER.fetch_add(1, Ordering::SeqCst);
        if !(*OCCUPIED_PORTS).contains(&candidate_port) {
            return candidate_port;
        }
    }
}
