pub mod registration;
pub mod voting_group;

use registration::{Delegations, MainnetRewardAddress, VotingRegistration};
use voting_group::VotingGroupAssigner;

use jormungandr_lib::{crypto::account::Identifier, interfaces::Value};
use serde::Deserialize;
use std::{borrow::Borrow, collections::BTreeMap, iter::Iterator, num::NonZeroU64};
use voting_hir::VoterHIR;

pub const CATALYST_VOTING_PURPOSE_TAG: u64 = 0;

#[derive(Deserialize, Clone, Debug)]
pub struct RawSnapshot(Vec<VotingRegistration>);

impl From<Vec<VotingRegistration>> for RawSnapshot {
    fn from(from: Vec<VotingRegistration>) -> Self {
        Self(from)
    }
}

/// Contribution to a voting key for some registration
#[derive(Clone, Debug, PartialEq)]
pub struct KeyContribution {
    pub reward_address: MainnetRewardAddress,
    pub value: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    // a raw public key is preferred so that we don't have to worry about discrimination when deserializing from
    // a CIP-36 compatible encoding
    inner: BTreeMap<Identifier, Vec<KeyContribution>>,
    stake_threshold: Value,
}

impl Snapshot {
    pub fn from_raw_snapshot(raw_snapshot: RawSnapshot, stake_threshold: Value) -> Self {
        Self {
            inner: raw_snapshot
                .0
                .into_iter()
                // Discard registrations with 0 voting power since they don't influence
                // snapshot anyway
                .filter(|reg| reg.voting_power >= std::cmp::max(stake_threshold, 1.into()))
                // TODO: add capability to select voting purpose for a snapshot.
                // At the moment Catalyst is the only one in use
                .filter(|reg| reg.voting_purpose == CATALYST_VOTING_PURPOSE_TAG)
                .fold(BTreeMap::new(), |mut acc, reg| {
                    let VotingRegistration {
                        reward_address,
                        delegations,
                        voting_power,
                        ..
                    } = reg;

                    match delegations {
                        Delegations::Legacy(vk) => {
                            acc.entry(vk).or_default().push(KeyContribution {
                                reward_address,
                                value: voting_power.into(),
                            });
                        }
                        Delegations::New(mut vks) => {
                            let voting_power = u64::from(voting_power);
                            let total_weights =
                                NonZeroU64::new(vks.iter().map(|(_, weight)| *weight as u64).sum());

                            let last = vks.pop().expect("CIP36 requires at least 1 delegation");
                            let others_total_vp = total_weights.map_or(0, |non_zero_total| {
                                vks.into_iter()
                                    .filter_map(|(vk, weight)| {
                                        NonZeroU64::new(
                                            (voting_power * weight as u64) / non_zero_total,
                                        )
                                        .map(|value| (vk, value))
                                    })
                                    .map(|(vk, value)| {
                                        acc.entry(vk).or_default().push(KeyContribution {
                                            reward_address: reward_address.clone(),
                                            value: value.get(),
                                        });
                                        value.get()
                                    })
                                    .sum::<u64>()
                            });
                            acc.entry(last.0).or_default().push(KeyContribution {
                                reward_address,
                                value: voting_power - others_total_vp,
                            });
                        }
                    };
                    acc
                }),
            stake_threshold,
        }
    }

    pub fn stake_threshold(&self) -> Value {
        self.stake_threshold
    }

    pub fn to_voter_hir(&self, voting_group_assigner: &impl VotingGroupAssigner) -> Vec<VoterHIR> {
        self.inner
            .iter()
            .map(|(voting_key, contribs)| VoterHIR {
                voting_key: voting_key.clone(),
                voting_power: contribs.iter().map(|c| c.value).sum::<u64>().into(),
                voting_group: voting_group_assigner.assign(voting_key),
            })
            .collect::<Vec<_>>()
    }

    pub fn voting_keys(&self) -> impl Iterator<Item = &Identifier> {
        self.inner.keys()
    }

    pub fn contributions_for_voting_key<I: Borrow<Identifier>>(
        &self,
        voting_public_key: I,
    ) -> Vec<KeyContribution> {
        self.inner
            .get(voting_public_key.borrow())
            .cloned()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chain_addr::{Discrimination, Kind};
    use jormungandr_lib::interfaces::{Address, InitialUTxO};
    use proptest::prelude::*;
    use test_strategy::proptest;

    impl Snapshot {
        pub fn to_block0_initials(&self, discrimination: Discrimination) -> Vec<InitialUTxO> {
            self.inner
                .iter()
                .map(|(vk, contribs)| {
                    let value: Value = contribs.iter().map(|c| c.value).sum::<u64>().into();
                    let address: Address =
                        chain_addr::Address(discrimination, Kind::Account(vk.to_inner().into()))
                            .into();
                    InitialUTxO { address, value }
                })
                .collect::<Vec<_>>()
        }
    }

    impl Arbitrary for RawSnapshot {
        type Parameters = ();
        type Strategy = BoxedStrategy<RawSnapshot>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            any::<Vec<VotingRegistration>>().prop_map(Self).boxed()
        }
    }

    #[proptest]
    fn test_threshold(raw: RawSnapshot, stake_threshold: u64, additional_reg: VotingRegistration) {
        let mut add = raw.clone();
        add.0.push(additional_reg.clone());
        assert_eq!(
            Snapshot::from_raw_snapshot(raw, stake_threshold.into())
                == Snapshot::from_raw_snapshot(add, stake_threshold.into()),
            additional_reg.voting_power < stake_threshold.into()
        );
    }

    impl Arbitrary for Snapshot {
        type Parameters = ();
        type Strategy = BoxedStrategy<Snapshot>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<RawSnapshot>(), 1..u64::MAX)
                .prop_map(|(raw_snapshot, threshold)| {
                    Self::from_raw_snapshot(raw_snapshot, threshold.into())
                })
                .boxed()
        }
    }

    // Test all voting power is distributed among delegated keys
    #[proptest]
    fn test_voting_power_all_distributed(reg: VotingRegistration) {
        let snapshot = Snapshot::from_raw_snapshot(vec![reg.clone()].into(), 0.into());
        let total_stake = snapshot
            .to_voter_hir(&|_vk: &Identifier| String::new())
            .into_iter()
            .map(|hir| u64::from(hir.voting_power))
            .sum::<u64>();
        assert_eq!(total_stake, u64::from(reg.voting_power))
    }

    #[proptest]
    fn test_non_catalyst_regs_are_ignored(mut reg: VotingRegistration) {
        reg.voting_purpose = 1;
        assert_eq!(
            Snapshot::from_raw_snapshot(vec![reg].into(), 0.into()),
            Snapshot::from_raw_snapshot(vec![].into(), 0.into()),
        )
    }

    #[test]
    fn test_distribution() {
        let mut raw_snapshot = Vec::new();
        let voting_pub_key_1 = Identifier::from_hex(&hex::encode([0; 32])).unwrap();
        let voting_pub_key_2 = Identifier::from_hex(&hex::encode([1; 32])).unwrap();

        let n = 10;
        for i in 1..=n {
            let delegations = Delegations::New(vec![
                (voting_pub_key_1.clone(), 1),
                (voting_pub_key_2.clone(), 1),
            ]);
            raw_snapshot.push(VotingRegistration {
                stake_public_key: String::new(),
                voting_power: i.into(),
                reward_address: String::new(),
                delegations,
                voting_purpose: 0,
            });
        }

        let snapshot = Snapshot::from_raw_snapshot(raw_snapshot.into(), 0.into());
        let vp_1: u64 = snapshot
            .contributions_for_voting_key(voting_pub_key_1)
            .into_iter()
            .map(|c| c.value)
            .sum();
        let vp_2: u64 = snapshot
            .contributions_for_voting_key(voting_pub_key_2)
            .into_iter()
            .map(|c| c.value)
            .sum();
        assert_eq!(vp_2 + vp_1, n * (n + 1) / 2);
        assert_eq!(vp_2 - vp_1, n / 2); // last key get the remainder during distribution
    }

    #[test]
    fn test_parsing() {
        let raw: RawSnapshot = serde_json::from_str(
            r#"[
            {
                "reward_address": "0xe1ffff2912572257b59dca84c965e4638a09f1524af7a15787eb0d8a46",
                "stake_public_key": "0xe7d6616840734686855ec80ee9658f5ead9e29e494ec6889a5d1988b50eb8d0f",
                "voting_power": 177689370111,
                "delegations": [
                    ["0xa6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663", 3],
                    ["0x00588e8e1d18cba576a4d35758069fe94e53f638b6faf7c07b8abd2bc5c5cdee", 1]
                ]
            }
        ]"#,
        ).unwrap();
        let snapshot = Snapshot::from_raw_snapshot(raw, 0.into());
        assert_eq!(
            snapshot.contributions_for_voting_key(
                Identifier::from_hex(
                    "a6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663"
                )
                .unwrap()
            )[0]
            .value,
            133267027583
        );
        assert_eq!(
            snapshot.contributions_for_voting_key(
                Identifier::from_hex(
                    "00588e8e1d18cba576a4d35758069fe94e53f638b6faf7c07b8abd2bc5c5cdee"
                )
                .unwrap()
            )[0]
            .value,
            44422342528
        );
    }
}
