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
        _parameters: SnapshotParameters,
    ) -> Result<RawSnapshotRequest, Error> {
        /*
        let (db_sync, _, _) = self
            .into_iter()
            .fold(
                MainnetNetworkBuilder::default(),
                MainnetNetworkBuilder::with,
            )
            .build();
        let _db = MockDbProvider::from(db_sync);
        */
        //let (outputs, _errs) =
        //    voting_tools_rs::voting_power(&db, VotingPowerArgs::default()).unwrap();
        //outputs.try_into_raw_snapshot_request(parameters)
        Ok(RawSnapshotRequest::default())
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

impl OutputsExtension for Vec<SnapshotEntry> {
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
                dreps: parameters.dreps,
            },
        })
    }
}

use jormungandr_lib::crypto::account::Identifier;
use mainnet_lib::wallet_state::{MainnetWalletState, TemplateError};
use mainnet_lib::SnapshotParameters;
use num_traits::ToPrimitive;
use snapshot_lib::registration::{
    Delegations as VotingDelegations, RewardAddress, StakeAddress, VotingRegistration,
};
use vit_servicing_station_lib::v0::endpoints::snapshot::RawSnapshotInput;
use voting_tools_rs::{SnapshotEntry, VotingKey, VotingPurpose};

/// Extensions for voting tools `Output` struct
pub trait OutputExtension {
    /// Converts to `VotingRegistration`
    ///
    /// # Errors
    ///
    /// At any internal error while creating a voting registration from output
    fn try_into_voting_registration(self) -> Result<VotingRegistration, Error>;
}

impl OutputExtension for SnapshotEntry {
    fn try_into_voting_registration(self) -> Result<VotingRegistration, Error> {
        Ok(VotingRegistration {
            stake_public_key: StakeAddress(self.stake_key.to_string()),
            voting_power: self
                .voting_power
                .to_u64()
                .ok_or_else(|| {
                    Error::CannotConvertFromOutput("cannot extract voting power".to_string())
                })?
                .into(),
            reward_address: RewardAddress(hex::encode(&self.rewards_address.0)),
            delegations: match self.voting_key {
                VotingKey::Direct(legacy) => VotingDelegations::Legacy(
                    Identifier::from_hex(&legacy.to_hex())
                        .expect("to_hex() always returns valid hex"),
                ),
                VotingKey::Delegated(delegated) => {
                    let mut new = vec![];
                    for (key, weight) in delegated {
                        new.push((
                            Identifier::from_hex(&key.to_hex())
                                .expect("to_hex() always returns valid hex"),
                            weight
                                .to_u32()
                                .expect("this tool expects voting powers that fit into a u32"),
                        ));
                    }
                    VotingDelegations::New(new)
                }
            },
            voting_purpose: Some(*self.voting_purpose.unwrap_or(VotingPurpose::CATALYST)),

            nonce: self.nonce,
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
