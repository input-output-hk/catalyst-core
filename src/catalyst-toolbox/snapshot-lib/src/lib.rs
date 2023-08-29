use chain_addr::Discrimination;
pub use fraction::Fraction;
use jormungandr_lib::{crypto::account::Identifier, interfaces::Value};
use registration::{
    serde_impl::IdentifierDef, Delegations, RewardAddress, StakeAddress, VotingRegistration,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    borrow::Borrow,
    collections::{BTreeMap, HashSet},
    iter::Iterator,
    num::NonZeroU64,
};
use thiserror::Error;
pub use voter_hir::VoterHIR;
pub use voter_hir::VotingGroup;
use voting_group::VotingGroupAssigner;

mod influence_cap;
pub mod registration;
pub mod sve;
mod voter_hir;
pub mod voting_group;

pub const CATALYST_VOTING_PURPOSE_TAG: u64 = 0;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawSnapshot(Vec<VotingRegistration>);

impl From<Vec<VotingRegistration>> for RawSnapshot {
    fn from(from: Vec<VotingRegistration>) -> Self {
        Self(from)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Dreps {
    #[serde(
        serialize_with = "serialize_identifiers",
        deserialize_with = "deserialize_identifiers"
    )]
    reps: HashSet<Identifier>,
}

impl From<HashSet<Identifier>> for Dreps {
    fn from(reps: HashSet<Identifier>) -> Self {
        Self { reps }
    }
}

fn serialize_identifiers<S: Serializer>(
    identifiers: &HashSet<Identifier>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let identifiers: Vec<_> = identifiers
        .iter()
        .map(|id| IdentifierDef(id.clone()))
        .collect();
    identifiers.serialize(serializer)
}

fn deserialize_identifiers<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<HashSet<Identifier>, D::Error> {
    Ok(Vec::<IdentifierDef>::deserialize(deserializer)?
        .into_iter()
        .map(|id| id.0)
        .collect())
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("insufficient number of voters to guarantee voting influence cap")]
    NotEnoughVoters,
    #[error("voting power overflow")]
    Overflow,
}

/// Contribution to a voting key for some registration
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyContribution {
    pub stake_public_key: StakeAddress,
    pub reward_address: RewardAddress,
    pub value: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotInfo {
    /// The values in the contributions are the original values in the registration transactions and
    /// thus retain the original proportions.
    /// However, it's possible that the sum of those values is greater than the voting power assigned in the
    /// VoterHIR, due to voting power caps or additional transformations.
    pub contributions: Vec<KeyContribution>,
    pub hir: VoterHIR,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    // a raw public key is preferred so that we don't have to worry about discrimination when deserializing from
    // a CIP-36 compatible encoding
    inner: BTreeMap<Identifier, SnapshotInfo>,
    stake_threshold: Value,
}

impl Snapshot {
    #[allow(clippy::missing_errors_doc)]
    pub fn from_raw_snapshot(
        raw_snapshot: RawSnapshot,
        stake_threshold: Value,
        cap: Fraction,
        voting_group_assigner: &impl VotingGroupAssigner,
        discrimination: Discrimination,
    ) -> Result<Self, Error> {
        let raw_contribs = raw_snapshot
            .0
            .into_iter()
            // Discard registrations with 0 voting power since they don't influence
            // snapshot anyway. But can not throw any others away, even if less than the stake threshold.
            .filter(|reg| reg.voting_power >= 1.into())
            // TODO: add capability to select voting purpose for a snapshot.
            // At the moment Catalyst is the only one in use
            .filter(|reg| {
                reg.voting_purpose.unwrap_or(CATALYST_VOTING_PURPOSE_TAG)
                    == CATALYST_VOTING_PURPOSE_TAG
            })
            .fold(BTreeMap::new(), |mut acc: BTreeMap<_, Vec<_>>, reg| {
                let VotingRegistration {
                    reward_address,
                    delegations,
                    voting_power,
                    stake_public_key,
                    ..
                } = reg;

                match delegations {
                    Delegations::Legacy(vk) => {
                        acc.entry(vk).or_default().push(KeyContribution {
                            stake_public_key,
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
                                    NonZeroU64::new((voting_power * weight as u64) / non_zero_total)
                                        .map(|value| (vk, value))
                                })
                                .map(|(vk, value)| {
                                    acc.entry(vk).or_default().push(KeyContribution {
                                        stake_public_key: stake_public_key.clone(),
                                        reward_address: reward_address.clone(),
                                        value: value.get(),
                                    });
                                    value.get()
                                })
                                .sum::<u64>()
                        });
                        acc.entry(last.0).or_default().push(KeyContribution {
                            stake_public_key,
                            reward_address,
                            value: voting_power - others_total_vp,
                        });
                    }
                };
                acc
            });
        let entries = raw_contribs
            .into_iter()
            .map(|(k, contributions)| SnapshotInfo {
                hir: VoterHIR {
                    voting_group: voting_group_assigner.assign(&k),
                    voting_key: k.clone(),
                    address: chain_addr::Address(
                        discrimination,
                        chain_addr::Kind::Account(k.to_inner().into()),
                    )
                    .into(),
                    voting_power: contributions.iter().map(|c| c.value).sum::<u64>().into(),
                },
                contributions,
            })
            // Because of multiple registrations to the same voting key,  we can only
            // filter once all registrations for the same key are known.
            // `stake_threshold` is the minimum stake for all registrations COMBINED.
            .filter(|entry| entry.hir.voting_power >= stake_threshold)
            .collect();

        Ok(Self {
            inner: Self::apply_voting_power_cap(entries, cap)?
                .into_iter()
                .map(|entry| (entry.hir.voting_key.clone(), entry))
                .collect(),
            stake_threshold,
        })
    }

    fn apply_voting_power_cap(
        voters: Vec<SnapshotInfo>,
        cap: Fraction,
    ) -> Result<Vec<SnapshotInfo>, Error> {
        Ok(influence_cap::cap_voting_influence(voters, cap)?
            .into_iter()
            .collect())
    }

    #[must_use]
    pub fn stake_threshold(&self) -> Value {
        self.stake_threshold
    }

    #[must_use]
    pub fn to_voter_hir(&self) -> Vec<VoterHIR> {
        self.inner
            .values()
            .map(|entry| entry.hir.clone())
            .collect::<Vec<_>>()
    }

    #[must_use]
    pub fn to_full_snapshot_info(&self) -> Vec<SnapshotInfo> {
        self.inner.values().cloned().collect()
    }

    pub fn voting_keys(&self) -> impl Iterator<Item = &Identifier> {
        self.inner.keys()
    }

    #[must_use]
    pub fn contributions_for_voting_key<I: Borrow<Identifier>>(
        &self,
        voting_public_key: I,
    ) -> Vec<KeyContribution> {
        self.inner
            .get(voting_public_key.borrow())
            .cloned()
            .map(|entry| entry.contributions)
            .unwrap_or_default()
    }
}

#[cfg(any(test, feature = "proptest"))]
pub mod tests {
    use super::*;
    use chain_addr::{Discrimination, Kind};
    use jormungandr_lib::interfaces::{Address, InitialUTxO};
    use proptest::prelude::*;
    use test_strategy::proptest;

    struct DummyAssigner;

    impl VotingGroupAssigner for DummyAssigner {
        fn assign(&self, _vk: &Identifier) -> String {
            String::new()
        }
    }

    impl Snapshot {
        #[must_use]
        pub fn to_block0_initials(&self, discrimination: Discrimination) -> Vec<InitialUTxO> {
            self.inner
                .iter()
                .map(|(vk, entry)| {
                    let value = entry.hir.voting_power;
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
    fn test_threshold(
        _raw: RawSnapshot,
        _stake_threshold: u64,
        _additional_reg: VotingRegistration,
    ) {
        let mut add = _raw.clone();
        add.0.push(_additional_reg.clone());
        assert_eq!(
            Snapshot::from_raw_snapshot(
                _raw,
                _stake_threshold.into(),
                Fraction::from(1u64),
                &DummyAssigner,
                Discrimination::Production,
            )
            .unwrap()
                == Snapshot::from_raw_snapshot(
                    add,
                    _stake_threshold.into(),
                    Fraction::from(1u64),
                    &DummyAssigner,
                    Discrimination::Production,
                )
                .unwrap(),
            _additional_reg.voting_power < _stake_threshold.into()
        );
    }

    impl Arbitrary for Snapshot {
        type Parameters = ();
        type Strategy = BoxedStrategy<Snapshot>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<RawSnapshot>(), 1..u64::MAX)
                .prop_map(|(raw_snapshot, threshold)| {
                    Self::from_raw_snapshot(
                        raw_snapshot,
                        threshold.into(),
                        Fraction::from(1),
                        &|_vk: &Identifier| String::new(),
                        Discrimination::Production,
                    )
                    .unwrap()
                })
                .boxed()
        }
    }

    // Test all voting power is distributed among delegated keys
    #[proptest]
    fn test_voting_power_all_distributed(_reg: VotingRegistration) {
        let snapshot = Snapshot::from_raw_snapshot(
            vec![_reg.clone()].into(),
            0.into(),
            Fraction::from(1),
            &|_vk: &Identifier| String::new(),
            Discrimination::Production,
        )
        .unwrap();
        let total_stake = snapshot
            .to_voter_hir()
            .into_iter()
            .map(|hir| u64::from(hir.voting_power))
            .sum::<u64>();
        assert_eq!(total_stake, u64::from(_reg.voting_power))
    }

    #[proptest]
    fn test_non_catalyst_regs_are_ignored(mut _reg: VotingRegistration) {
        _reg.voting_purpose = Some(1);
        assert_eq!(
            Snapshot::from_raw_snapshot(
                vec![_reg].into(),
                0.into(),
                Fraction::from(1u64),
                &DummyAssigner,
                Discrimination::Production,
            )
            .unwrap(),
            Snapshot::from_raw_snapshot(
                vec![].into(),
                0.into(),
                Fraction::from(1u64),
                &DummyAssigner,
                Discrimination::Production,
            )
            .unwrap(),
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
                stake_public_key: StakeAddress(String::new()),
                voting_power: i.into(),
                reward_address: RewardAddress(String::new()),
                delegations,
                voting_purpose: Some(0),
                nonce: 0,
            });
        }

        let snapshot = Snapshot::from_raw_snapshot(
            raw_snapshot.into(),
            0.into(),
            Fraction::from(1u64),
            &DummyAssigner,
            Discrimination::Production,
        )
        .unwrap();
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
    fn test_raw_snapshot_parsing() {
        let raw: RawSnapshot = serde_json::from_str(
            r#"[
            {
                "rewards_address": "0xe1ffff2912572257b59dca84c965e4638a09f1524af7a15787eb0d8a46",
                "stake_public_key": "0xe7d6616840734686855ec80ee9658f5ead9e29e494ec6889a5d1988b50eb8d0f",
                "voting_power": 177689370111,
                "delegations": [
                    ["0xa6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663", 3],
                    ["0x00588e8e1d18cba576a4d35758069fe94e53f638b6faf7c07b8abd2bc5c5cdee", 1]
                ]
            }
        ]"#,
        ).unwrap();
        let snapshot = Snapshot::from_raw_snapshot(
            raw,
            0.into(),
            Fraction::from(1u64),
            &DummyAssigner,
            Discrimination::Production,
        )
        .unwrap();
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

    #[test]
    fn test_drep_info_parsing() {
        let dreps: Dreps = serde_json::from_str(r#"{
            "reps": ["0x00588e8e1d18cba576a4d35758069fe94e53f638b6faf7c07b8abd2bc5c5cdee", "0xa6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663", "0xa6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663"]
        }"#).unwrap();

        assert_eq!(dreps.reps.len(), 2);
        assert!(dreps.reps.contains(
            &Identifier::from_hex(
                "00588e8e1d18cba576a4d35758069fe94e53f638b6faf7c07b8abd2bc5c5cdee",
            )
            .unwrap()
        ));
        assert!(dreps.reps.contains(
            &Identifier::from_hex(
                "a6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663",
            )
            .unwrap()
        ));
    }
}
