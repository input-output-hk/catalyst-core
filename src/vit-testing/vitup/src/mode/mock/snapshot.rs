use crate::config::{SnapshotError, SnapshotInitials};
use hersir::builder::Wallet;
use jormungandr_lib::crypto::account::Identifier;
use proptest::{arbitrary::Arbitrary, prelude::*, strategy::BoxedStrategy};
use snapshot_lib::VoterHIR;
use std::collections::BTreeMap;
use thor::WalletAlias;
use vit_servicing_station_lib::db::models::snapshot::{Contribution, Voter};

#[derive(Debug, Default)]
pub struct VoterSnapshot {
    /// key: Tag - a unique identifier of the current snapshot
    /// value: Timestamp - for the latest update of the current snapshot
    snapshot_tags: BTreeMap<String, i64>,
    voters: Vec<Voter>,
    contributions: Vec<Contribution>,
}

impl VoterSnapshot {
    pub fn from_config_or_default(
        wallets: Vec<(WalletAlias, &Wallet)>,
        initials: &Option<SnapshotInitials>,
    ) -> Result<Self, SnapshotError> {
        let mut voters = vec![];
        let mut contributions = vec![];
        let mut snapshot_tags = BTreeMap::new();

        if let Some(initials) = initials {
            snapshot_tags.insert(initials.tag.to_string(), epoch_now());
            let voter_hirs = initials.as_voters_hirs(wallets)?;

            for voter_hir in voter_hirs {
                voters.push(Voter {
                    voting_key: voter_hir.voting_key.to_bech32_str(),
                    voting_power: u64::from(voter_hir.voting_power) as i64,
                    voting_group: voter_hir.voting_group.to_string(),
                    snapshot_tag: initials.tag.to_string(),
                });

                contributions.push(Contribution {
                    stake_public_key: "mock".to_string(),
                    reward_address: "mock".to_string(),
                    value: u64::from(voter_hir.voting_power) as i64,
                    voting_key: voter_hir.voting_key.to_bech32_str(),
                    voting_group: voter_hir.voting_group.to_string(),
                    snapshot_tag: initials.tag.to_string(),
                });
            }
        }

        Ok(Self {
            snapshot_tags,
            voters,
            contributions,
        })
    }

    pub fn tags(&self) -> Vec<String> {
        self.snapshot_tags.keys().cloned().collect()
    }

    pub fn put_snapshot_tag(&mut self, tag: String, timestamp: i64) {
        self.snapshot_tags.insert(tag, timestamp);
    }

    pub fn snapshot_by_tag(&self, tag: impl Into<String>) -> Option<&i64> {
        self.snapshot_tags.get(&tag.into())
    }

    pub fn contributions_by_stake_public_key_and_snapshot_tag(
        &self,
        stake_public_key: &str,
        tag: &str,
    ) -> Vec<&Contribution> {
        self.contributions
            .iter()
            .filter(|v| v.stake_public_key == stake_public_key && v.snapshot_tag == tag)
            .collect()
    }

    pub fn total_voting_power_by_voting_group_and_snapshot_tag(
        &self,
        voting_group: &str,
        snapshot_tag: &str,
    ) -> i64 {
        self.voters
            .iter()
            .filter(|v| v.voting_group == voting_group && v.snapshot_tag == snapshot_tag)
            .map(|v| v.voting_power)
            .sum()
    }

    pub fn contributions_by_voting_key_and_voter_group_and_snapshot_tag(
        &self,
        voting_key: &str,
        voting_group: &str,
        snapshot_tag: &str,
    ) -> Vec<&Contribution> {
        self.contributions
            .iter()
            .filter(|v| {
                v.voting_key == voting_key
                    && v.voting_group == voting_group
                    && v.snapshot_tag == snapshot_tag
            })
            .collect()
    }

    pub fn voters_by_voting_key_and_snapshot_tag(
        &self,
        voting_key: &str,
        snapshot_tag: &str,
    ) -> Vec<&Voter> {
        self.voters
            .iter()
            .filter(|v| v.voting_key == voting_key && v.snapshot_tag == snapshot_tag)
            .collect()
    }

    pub fn insert_voters(&mut self, voters: Vec<Voter>) {
        for voter in voters {
            if let Some(idx) = self
                .voters
                .iter()
                .enumerate()
                .find(|(_, x)| {
                    x.voting_key == voter.voting_key
                        && x.snapshot_tag == voter.snapshot_tag
                        && x.voting_group == voter.voting_group
                })
                .map(|(idx, _)| idx)
            {
                let _ = std::mem::replace(&mut self.voters[idx], voter);
            } else {
                self.voters.push(voter)
            }
        }
    }

    pub fn insert_contributions(&mut self, contributions: Vec<Contribution>) {
        for contribution in contributions {
            if let Some(idx) = self
                .contributions
                .iter()
                .enumerate()
                .find(|(_, x)| {
                    x.stake_public_key == contribution.stake_public_key
                        && x.voting_key == contribution.voting_key
                        && x.voting_group == contribution.voting_group
                        && x.snapshot_tag == contribution.snapshot_tag
                })
                .map(|(idx, _)| idx)
            {
                let _ = std::mem::replace(&mut self.contributions[idx], contribution);
            } else {
                self.contributions.push(contribution)
            }
        }
    }
}

#[derive(Debug)]
struct ArbitraryVoterHIR(VoterHIR);

impl Arbitrary for ArbitraryVoterHIR {
    type Parameters = Option<String>;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        if let Some(voting_group) = args {
            any::<([u8; 32], u64)>()
                .prop_map(move |(key, voting_power)| {
                    Self(VoterHIR {
                        voting_key: Identifier::from_hex(&hex::encode(key)).unwrap(),
                        voting_power: voting_power.into(),
                        voting_group: voting_group.clone(),
                    })
                })
                .boxed()
        } else {
            any::<([u8; 32], u64, String)>()
                .prop_map(|(key, voting_power, voting_group)| {
                    Self(VoterHIR {
                        voting_key: Identifier::from_hex(&hex::encode(key)).unwrap(),
                        voting_power: voting_power.into(),
                        voting_group,
                    })
                })
                .boxed()
        }
    }
}

use std::time::{SystemTime, UNIX_EPOCH};
fn epoch_now() -> i64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let ms = since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000;

    ms as i64
}

impl Arbitrary for VoterSnapshot {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        let tags = vec![
            String::from("latest"),
            String::from("fund8"),
            String::from("nightly"),
        ];
        any_with::<(Vec<ArbitraryVoterHIR>, Vec<ArbitraryVoterHIR>, usize)>((
            (Default::default(), Some("direct".to_string())),
            (Default::default(), Some("dreps".to_string())),
            (),
        ))
        .prop_map(move |(dreps, voters, random)| {
            let mut snapshot_voters = vec![];

            snapshot_voters.extend(dreps.iter().map(|drep| Voter {
                voting_key: drep.0.voting_key.to_bech32_str(),
                voting_power: u64::from(drep.0.voting_power) as i64,
                voting_group: drep.0.voting_group.to_string(),
                snapshot_tag: tags[random % tags.len()].clone(),
            }));

            snapshot_voters.extend(voters.iter().map(|voter| Voter {
                voting_key: voter.0.voting_key.to_bech32_str(),
                voting_power: u64::from(voter.0.voting_power) as i64,
                voting_group: voter.0.voting_group.to_string(),
                snapshot_tag: tags[random % tags.len()].clone(),
            }));

            let mut contributions = vec![];

            contributions.extend(snapshot_voters.iter().map(|voter| Contribution {
                stake_public_key: voter.voting_key.to_string(),
                reward_address: voter.voting_key.to_string(),
                value: voter.voting_power,
                voting_key: voter.voting_key.clone(),
                voting_group: voter.voting_group.clone(),
                snapshot_tag: voter.snapshot_tag.clone(),
            }));

            Self {
                snapshot_tags: tags.iter().cloned().map(|t| (t, epoch_now())).collect(),
                voters: snapshot_voters,
                contributions,
            }
        })
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tags() {
        let mut voter_snapshot = VoterSnapshot::default();

        voter_snapshot.put_snapshot_tag("a".to_string(), epoch_now());
        voter_snapshot.put_snapshot_tag("b".to_string(), epoch_now());
        voter_snapshot.put_snapshot_tag("c".to_string(), epoch_now());
        assert_eq!(
            &[String::from("a"), String::from("b"), String::from("c")],
            voter_snapshot.tags().as_slice()
        );
    }
}
