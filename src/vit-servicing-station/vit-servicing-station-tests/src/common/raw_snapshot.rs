use std::default;

use chain_impl_mockchain::testing::TestGen;
use itertools::Itertools;
use rand::Rng;
use serde::{Deserialize, Serialize};
use snapshot_lib::registration::VotingRegistration;
use snapshot_lib::{
    voting_group::{DEFAULT_DIRECT_VOTER_GROUP, DEFAULT_REPRESENTATIVE_GROUP},
    KeyContribution, SnapshotInfo, VoterHIR, CATALYST_VOTING_PURPOSE_TAG,
};
use time::OffsetDateTime;
use vit_servicing_station_lib::v0::endpoints::snapshot::RawSnapshotInput;

#[derive(Debug)]
pub struct RawSnapshot {
    pub tag: String,
    pub content: RawSnapshotInput,
}

impl Default for RawSnapshot {
    fn default() -> RawSnapshot {
        RawSnapshotBuilder::default().build()
    }
}

#[derive(Debug)]
pub struct RawSnapshotBuilder {
    tag: String,
    update_timestamp: i64,
    min_stake_threshold: Value,
    voting_power_cap: Fraction,
    direct_voters_group: Option<String>,
    representatives_group: Option<String>,
    voting_registrations_count: u32,
}

impl Default for RawSnapshotBuilder {
    fn default() -> RawSnapshotBuilder {
        Self {
            tag: "daily".to_string(),
            update_timestamp: OffsetDateTime::now_utc().unix_timestamp(),
            min_stake_threshold: 0.into(),
            voting_power_cap: 100.into(),
            direct_voters_group: DEFAULT_DIRECT_VOTER_GROUP,
            representatives_group: DEFAULT_REPRESENTATIVE_GROUP,
            voting_registrations_count: 2,
        }
    }
}

impl RawSnapshotBuilder {
    pub fn build(self) -> RawSnapshot {
        let voting_pub_key_1 = Identifier::from_hex(&hex::encode([0; 32])).unwrap();
        let voting_pub_key_2 = Identifier::from_hex(&hex::encode([1; 32])).unwrap();

        RawSnapshot {
            content: RawSnapshotInput {
                snapshot: std::iter::from_fn(|| {
                    Some(VotingRegistration {
                        stake_public_key:
                            "0x00588e8e1d18cba576a4d35758069fe94e53f638b6faf7c07b8abd2bc5c5cdee"
                                .to_string(),
                        voting_power: 1.into(),
                        reward_address:
                            "0x00588e8e1d18cba576a4d35758069fe94e53f638b6faf7c07b8abd2bc5c5cdee"
                                .to_string(),
                        delegations: Delegations::New(vec![
                            (voting_pub_key_1.clone(), 1),
                            (voting_pub_key_2.clone(), 1),
                        ]),
                        voting_purpose: CATALYST_VOTING_PURPOSE_TAG,
                    })
                })
                .take(self.voting_registrations_count)
                .collect(),
                update_timestamp: self.update_timestamp,
                min_stake_threshold: self.min_stake_threshold,
                voting_power_cap: self.voting_power_cap,
                direct_voters_group: self.direct_voters_group,
                representatives_group: self.direct_voters_group,
            },
            tag: self.tag,
        }
    }
}
