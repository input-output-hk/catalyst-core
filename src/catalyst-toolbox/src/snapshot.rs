use std::collections::HashMap;

use chain_addr::{Discrimination, Kind};
use jormungandr_lib::crypto::account::Identifier;
use jormungandr_lib::interfaces::{Address, Initial, InitialUTxO, Stake, Value};
use serde::{de::Error, Deserialize, Deserializer};
use std::iter::Iterator;

pub type MainnetRewardAddress = String;
pub type MainnetStakeAddress = String;

#[derive(Deserialize, Clone, Debug)]
pub struct CatalystRegistration {
    pub stake_public_key: MainnetStakeAddress,
    pub voting_power: Stake,
    #[serde(deserialize_with = "reward_addr_from_hex")]
    pub reward_address: MainnetRewardAddress,
    #[serde(deserialize_with = "identifier_from_hex")]
    pub voting_public_key: Identifier,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RawSnapshot(Vec<CatalystRegistration>);

#[derive(Clone, Debug)]
pub struct Snapshot {
    // a raw public key is preferred so that we don't have to worry about discrimination when deserializing from
    // a CIP-15 compatible encoding
    inner: HashMap<Identifier, Vec<CatalystRegistration>>,
    stake_threshold: Stake,
}

impl Snapshot {
    pub fn from_raw_snapshot(raw_snapshot: RawSnapshot, stake_threshold: Stake) -> Self {
        Self {
            inner: raw_snapshot
                .0
                .into_iter()
                .filter(|reg| reg.voting_power >= stake_threshold)
                .fold(HashMap::new(), |mut acc, reg| {
                    acc.entry(reg.voting_public_key.clone())
                        .or_default()
                        .push(reg);
                    acc
                }),
            stake_threshold,
        }
    }

    pub fn stake_threshold(&self) -> Stake {
        self.stake_threshold
    }

    pub fn to_block0_initials(&self, discrimination: Discrimination) -> Initial {
        Initial::Fund(
            self.inner
                .iter()
                .map(|(vk, regs)| {
                    let value: Value = regs
                        .iter()
                        .map(|reg| u64::from(reg.voting_power))
                        .sum::<u64>()
                        .into();
                    let address: Address =
                        chain_addr::Address(discrimination, Kind::Account(vk.to_inner().into()))
                            .into();
                    InitialUTxO { address, value }
                })
                .collect::<Vec<_>>(),
        )
    }

    pub fn voting_keys(&self) -> impl Iterator<Item = &Identifier> {
        self.inner.keys()
    }

    pub fn registrations_for_voting_key<I: Into<Identifier>>(
        &self,
        voting_public_key: I,
    ) -> Vec<CatalystRegistration> {
        let voting_public_key: Identifier = voting_public_key.into();
        self.inner
            .get(&voting_public_key)
            .cloned()
            .unwrap_or_default()
    }
}

fn identifier_from_hex<'de, D>(deserializer: D) -> Result<Identifier, D::Error>
where
    D: Deserializer<'de>,
{
    let hex = String::deserialize(deserializer)?;
    Identifier::from_hex(hex.trim_start_matches("0x"))
        .map_err(|e| D::Error::custom(format!("invalid public key {}", e)))
}

fn reward_addr_from_hex<'de, D>(deserializer: D) -> Result<MainnetRewardAddress, D::Error>
where
    D: Deserializer<'de>,
{
    use bech32::ToBase32;
    let bytes = hex::decode(String::deserialize(deserializer)?.trim_start_matches("0x"))
        .map_err(|e| D::Error::custom(format!("invalid hex string: {}", e)))?;
    bech32::encode("stake", &bytes.to_base32(), bech32::Variant::Bech32)
        .map_err(|e| D::Error::custom(format!("bech32 encoding failed: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bech32::ToBase32;
    use chain_crypto::{Ed25519, SecretKey};
    use proptest::prelude::*;
    use test_strategy::proptest;

    impl Arbitrary for CatalystRegistration {
        type Parameters = ();
        type Strategy = BoxedStrategy<CatalystRegistration>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<([u8; 32], [u8; 32], [u8; 32])>(), 0..45_000_000u64)
                .prop_map(|((stake_key, rewards_addr, voting_key), vp)| {
                    let stake_public_key = hex::encode(stake_key);
                    let reward_address =
                        bech32::encode("stake", &rewards_addr.to_base32(), bech32::Variant::Bech32)
                            .unwrap();
                    let voting_public_key = <SecretKey<Ed25519>>::from_binary(&voting_key)
                        .expect("every binary sequence is a valid secret key")
                        .to_public()
                        .into();
                    let voting_power: Stake = vp.into();
                    CatalystRegistration {
                        stake_public_key,
                        voting_power,
                        reward_address,
                        voting_public_key,
                    }
                })
                .boxed()
        }
    }

    impl Arbitrary for RawSnapshot {
        type Parameters = ();
        type Strategy = BoxedStrategy<RawSnapshot>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            any::<Vec<CatalystRegistration>>()
                .prop_map(|regs| Self(regs))
                .boxed()
        }
    }

    #[proptest]
    fn test_threshold(raw: RawSnapshot, stake_threshold: u64) {
        let snapshot = Snapshot::from_raw_snapshot(raw, stake_threshold.into());
        assert!(!snapshot
            .inner
            .values()
            .flat_map(|regs| regs.iter().map(|reg| u64::from(reg.voting_power)))
            .any(|voting_power| voting_power < stake_threshold));
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

    impl From<Vec<CatalystRegistration>> for RawSnapshot {
        fn from(from: Vec<CatalystRegistration>) -> Self {
            Self(from)
        }
    }

    #[test]
    fn test_parsing() {
        let raw: RawSnapshot = serde_json::from_str(
            r#"[
            {
                "reward_address": "0xe1ffff2912572257b59dca84c965e4638a09f1524af7a15787eb0d8a46",
                "stake_public_key": "0xe7d6616840734686855ec80ee9658f5ead9e29e494ec6889a5d1988b50eb8d0f",
                "voting_power": 177689370111,
                "voting_public_key": "0xc21ddb4abb04bd5ce21091eef1676e44889d806e6e1a6a9a7dc25c0eba54cc33"
            },
            {
                "reward_address": "0xe1fffc8bcb1578a15413bf11413639fa270a9ffa36d9a0c4d2c93536fe",
                "stake_public_key": "0x2f9a90d87321a255efd038fea5df2a2349ea2c32fa584b73f2a46f655f235919",
                "voting_power": 9420156337,
                "voting_public_key": "0x3f656a1ba4ea8b33c81961fee6f15f09600f024435b1a7ada1e5b77b03a41a6d"
            },
            {
                "reward_address": "0xe1fff825e1bf009d35d9160f6340250b581f5d37c17538e960c0410b20",
                "stake_public_key": "0x66ae1553036548b99b93c783811bb281be5a196a12d950bda4ac9b83630afbd1",
                "voting_power": 82168168290,
                "voting_public_key": "0x125860fc4870bb480d1d2a97f101e1c5c845c0222400fdaba7bcca93e79bd66e"
            }
        ]"#,
        ).unwrap();
        Snapshot::from_raw_snapshot(raw, 0.into());
    }
}
