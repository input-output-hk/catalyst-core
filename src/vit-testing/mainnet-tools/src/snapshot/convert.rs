use std::time::{SystemTime, UNIX_EPOCH};
use vit_servicing_station_tests::common::raw_snapshot::RawSnapshot as RawSnapshotRequest;

/// Extensions for `MainnetWalletStateExtension` struct
pub trait MainnetWalletStateExtension {
    /// Converts to `RawSnapshotRequest`
    ///
    /// # Errors
    ///
    /// At any internal error while creating a snapshot
    fn try_into_raw_snapshot_request(
        self,
        parameters: SnapshotParameters,
    ) -> Result<RawSnapshotRequest, Error>;
}

impl MainnetWalletStateExtension for Vec<MainnetWalletState> {
    fn try_into_raw_snapshot_request(
        self,
        parameters: SnapshotParameters,
    ) -> Result<RawSnapshotRequest, Error> {
        let (db_sync, _, _) = self
            .into_iter()
            .fold(
                MainnetNetworkBuilder::default(),
                MainnetNetworkBuilder::with,
            )
            .build();
        let db = MockDbProvider::from(db_sync);
        let outputs = voting_tools_rs::voting_power(&db, None, None).unwrap();
        outputs.try_into_raw_snapshot_request(parameters)
    }
}

/// Extensions for collection of voting tools `Output` struct
pub trait OutputsExtension {
    /// Converts to `RawSnapshotRequest`
    ///
    /// # Errors
    ///
    /// At any internal error while creating a snapshot
    fn try_into_raw_snapshot_request(
        self,
        parameters: SnapshotParameters,
    ) -> Result<RawSnapshotRequest, Error>;
}

impl OutputsExtension for Vec<Output> {
    fn try_into_raw_snapshot_request(
        self,
        parameters: SnapshotParameters,
    ) -> Result<RawSnapshotRequest, Error> {
        let mut regs = Vec::new();

        for output in self {
            regs.push(output.try_into_voting_registration()?);
        }

        Ok(RawSnapshotRequest {
            tag: parameters.tag.clone(),
            content: RawSnapshotInput {
                snapshot: regs.into(),
                update_timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
                    .try_into()?,
                min_stake_threshold: parameters.min_stake_threshold,
                voting_power_cap: parameters.voting_power_cap,
                direct_voters_group: parameters.direct_voters_group.clone(),
                representatives_group: parameters.representatives_group,
            },
        })
    }
}

use jormungandr_lib::crypto::account::Identifier;
use mainnet_lib::wallet_state::{MainnetWalletState, TemplateError};
use mainnet_lib::{MainnetNetworkBuilder, SnapshotParameters};
use num_traits::ToPrimitive;
use snapshot_lib::registration::{Delegations as VotingDelegations, VotingRegistration};
use vit_servicing_station_lib::v0::endpoints::snapshot::RawSnapshotInput;
use voting_tools_rs::test_api::MockDbProvider;
use voting_tools_rs::{Delegations, Output};

/// Extensions for voting tools `Output` struct
pub trait OutputExtension {
    /// Converts to `VotingRegistration`
    ///
    /// # Errors
    ///
    /// At any internal error while creating a voting registration from output
    fn try_into_voting_registration(self) -> Result<VotingRegistration, Error>;
}

impl OutputExtension for Output {
    fn try_into_voting_registration(self) -> Result<VotingRegistration, Error> {
        Ok(VotingRegistration {
            stake_public_key: self.stake_public_key.to_string(),
            voting_power: self
                .voting_power
                .to_u64()
                .ok_or_else(|| {
                    Error::CannotConvertFromOutput("cannot extract voting power".to_string())
                })?
                .into(),
            reward_address: self.rewards_address.to_string(),
            delegations: match self.delegations {
                Delegations::Legacy(legacy) => VotingDelegations::Legacy(
                    Identifier::from_hex(legacy.trim_start_matches("0x"))
                        .map_err(|e| Error::CannotConvertFromOutput(e.to_string()))?,
                ),
                Delegations::Delegated(delegated) => {
                    let mut new = vec![];
                    for (key, weight) in delegated {
                        new.push((
                            Identifier::from_hex(key.trim_start_matches("0x"))
                                .map_err(|e| Error::CannotConvertFromOutput(e.to_string()))?,
                            weight,
                        ));
                    }
                    VotingDelegations::New(new)
                }
            },
            voting_purpose: *self.voting_purpose,
        })
    }
}

/// Conversion related errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Errors derived from time conversion
    #[error("cannot convert time milliseconds since start of the epoch")]
    TimeConversion(#[from] std::num::TryFromIntError),
    /// Error related to incorrect conversion
    #[error("cannot convert voting registration from voting tools output due to: {0}")]
    CannotConvertFromOutput(String),
    /// Error related to snapshot conversion
    #[error(transparent)]
    Snapshot(#[from] snapshot_lib::Error),
    /// Error related to building mock snapshot
    #[error(transparent)]
    Template(#[from] TemplateError),
}
