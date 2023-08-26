use jormungandr_lib::crypto::account::Identifier;
use jormungandr_lib::interfaces::Value;
use serde::{de::Error as _, ser::Error as _, Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

const MAINNET_PAYMENT_PREFIX: &str = "addr";
const TESTNET_PAYMENT_PREFIX: &str = "addr_test";
const MAINNET_STAKE_PREFIX: &str = "stake";
const TESTNET_STAKE_PREFIX: &str = "stake_test";

#[derive(Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RewardAddress(pub String);

impl Deref for RewardAddress {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RewardAddress {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct StakeAddress(pub String);

impl Deref for StakeAddress {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StakeAddress {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// The voting registration/delegation format as introduced in CIP-36,
/// which is a generalization of CIP-15, allowing to distribute
/// voting power among multiple keys in a single transaction and
/// to tag the purpose of the vote.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)]
pub struct VotingRegistration {
    pub stake_public_key: StakeAddress,
    pub voting_power: Value,
    /// Shelley address discriminated for the same network this transaction is submitted to.
    #[serde(rename = "rewards_address")]
    pub reward_address: RewardAddress,
    pub delegations: Delegations,
    /// 0 = Catalyst, assumed 0 for old legacy registrations
    #[serde(default)]
    pub voting_purpose: Option<u64>,

    #[serde(default)]
    pub nonce: u64,
}

impl VotingRegistration {
    #[must_use]
    pub fn is_legacy(&self) -> bool {
        matches!(self.delegations, Delegations::Legacy(_))
    }

    #[must_use]
    pub fn is_new(&self) -> bool {
        !self.is_legacy()
    }
}

/// To allow backward compatibility and avoid requiring existing users to
/// re-register we still consider valid old CIP-15 registrations, with the
/// simple correspondence between the two described in CIP-36.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Delegations {
    /// Tuples of (voting key, weight)
    New(Vec<(Identifier, u32)>),
    Legacy(Identifier),
}

pub mod serde_impl {
    use super::*;
    use chain_crypto::{Ed25519, PublicKey};
    use serde::{
        de::{self, Deserialize, Deserializer, SeqAccess, Visitor},
        Serialize, Serializer,
    };
    use std::fmt;

    pub struct IdentifierDef(pub(crate) Identifier);
    struct VotingKeyVisitor;

    impl Serialize for IdentifierDef {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            if serializer.is_human_readable() {
                serializer.serialize_str(&format!("0x{}", self.0.to_hex()))
            } else {
                serializer.serialize_bytes(self.0.as_ref().as_ref())
            }
        }
    }

    impl Serialize for Delegations {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match self {
                Self::Legacy(key) => IdentifierDef(key.clone()).serialize(serializer),
                Self::New(vec) => vec
                    .iter()
                    .map(|(vk, weight)| (IdentifierDef(vk.clone()), weight))
                    .collect::<Vec<_>>()
                    .serialize(serializer),
            }
        }
    }

    impl<'de> Visitor<'de> for VotingKeyVisitor {
        type Value = Identifier;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a voting key as described in CIP-36")
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            Identifier::from_hex(v.trim_start_matches("0x"))
                .map_err(|e| E::custom(format!("invalid voting key {}: {}", v, e)))
        }

        fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
            <PublicKey<Ed25519>>::from_binary(v)
                .map_err(|e| E::custom(format!("invalid voting key: {}", e)))
                .map(Self::Value::from)
        }
    }

    impl<'de> Deserialize<'de> for IdentifierDef {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            if deserializer.is_human_readable() {
                deserializer.deserialize_str(VotingKeyVisitor).map(Self)
            } else {
                deserializer.deserialize_bytes(VotingKeyVisitor).map(Self)
            }
        }
    }

    impl<'de> Deserialize<'de> for Delegations {
        fn deserialize<D>(deserializer: D) -> Result<Delegations, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct DelegationsVisitor;

            impl<'de> Visitor<'de> for DelegationsVisitor {
                type Value = Delegations;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("delegations as described in CIP-36")
                }

                // If we have to visit a str that means we are trying to deserialize the legacy
                // variant of the enum with a single voting key in hex format
                fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                    Ok(Self::Value::Legacy(VotingKeyVisitor.visit_str(v)?))
                }

                // Same thing for bytes
                fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
                    Ok(Self::Value::Legacy(VotingKeyVisitor.visit_bytes(v)?))
                }

                // If we are visiting a sequence of values that means we are trying to deserialize
                // the new variant of the enum
                fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
                where
                    V: SeqAccess<'de>,
                {
                    let mut vks = Vec::with_capacity(seq.size_hint().unwrap_or(1));
                    while let Some((vk, weight)) = seq.next_element::<(IdentifierDef, u32)>()? {
                        vks.push((vk.0, weight));
                    }
                    if vks.is_empty() {
                        return Err(V::Error::custom("expected at least one delegation"));
                    }

                    Ok(Self::Value::New(vks))
                }
            }
            // This is to support untagged variants (i.e. both legacy and new delegations format) without
            // any overhead since knowing the data type alone is enough to discriminate the variants.
            //
            // A safer way to do this would be to try parsing each variant instead of relying on the
            // deserializer to know the data type, which is not available in some formats like bincode,
            deserializer.deserialize_any(DelegationsVisitor)
        }
    }

    impl Serialize for RewardAddress {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            enum AddrType {
                Shelley,
                Stake,
            }
            enum NetType {
                Mainnet,
                Testnet,
            }
            // Following cip-0019 specification https://github.com/cardano-foundation/CIPs/blob/master/CIP-0019/README.md of the addresses formats
            use bech32::ToBase32;
            let bytes = hex::decode(self.trim_start_matches("0x"))
                .map_err(|e| S::Error::custom(format!("invalid hex string: {}", e)))?;

            let addr_prefix = bytes
                .first()
                .ok_or_else(|| S::Error::custom("invalid address format"))?;

            // Shelley addrs: 0x0?, 0x1?, 0x2?, 0x3?, 0x4?, 0x5?, 0x6?, 0x7?
            // Stake addrs: 0xE?, 0xF?
            let addr_type = addr_prefix >> 4 & 0xf;
            // 0 or 1 are valid addrs in the following cases:
            // type = 0x0 -  Testnet network
            // type = 0x1 -  Mainnet network
            let addr_net = addr_prefix & 0xf;

            let addr_type = match addr_type {
                // Shelley
                0x0 | 0x1 | 0x2 | 0x3 | 0x4 | 0x5 | 0x6 | 0x7 => AddrType::Shelley,
                // Stake
                0xf | 0xe => AddrType::Stake,
                _ => {
                    return Err(S::Error::custom(format!(
                        "invalid address format, incorrect addr type: {}",
                        addr_type
                    )))
                }
            };

            let addr_net = match addr_net {
                // Mainnet
                0x1 => NetType::Mainnet,
                // Testnet
                0x0 => NetType::Testnet,
                _ => {
                    return Err(S::Error::custom(format!(
                        "invalid address format, incorrect network tag: {}",
                        addr_net
                    )))
                }
            };

            let prefix = match (addr_type, addr_net) {
                (AddrType::Shelley, NetType::Mainnet) => MAINNET_PAYMENT_PREFIX,
                (AddrType::Stake, NetType::Mainnet) => MAINNET_STAKE_PREFIX,
                (AddrType::Shelley, NetType::Testnet) => TESTNET_PAYMENT_PREFIX,
                (AddrType::Stake, NetType::Testnet) => TESTNET_STAKE_PREFIX,
            };

            bech32::encode(prefix, bytes.to_base32(), bech32::Variant::Bech32)
                .map_err(|e| S::Error::custom(format!("bech32 encoding failed: {}", e)))?
                .serialize(serializer)
        }
    }
}

#[cfg(any(test, feature = "proptest"))]
mod tests {
    use super::*;
    use chain_crypto::{Ed25519, SecretKey};
    use proptest::collection::vec;
    use proptest::prelude::*;
    #[cfg(test)]
    use serde_test::{assert_de_tokens, Configure, Token};
    #[cfg(test)]
    use test_strategy::proptest;

    impl Arbitrary for Delegations {
        type Parameters = ();
        type Strategy = BoxedStrategy<Delegations>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            vec(any::<([u8; 32], u32)>(), 1..10) // Default is 0..100
                .prop_map(|vec| {
                    Delegations::New(
                        vec.into_iter()
                            .map(|(key, weight)| {
                                (
                                    <SecretKey<Ed25519>>::from_binary(&key)
                                        .unwrap()
                                        .to_public()
                                        .into(),
                                    weight,
                                )
                            })
                            .collect(),
                    )
                })
                .boxed()
                .prop_union(
                    any::<[u8; 32]>()
                        .prop_map(|key| {
                            Delegations::Legacy(
                                <SecretKey<Ed25519>>::from_binary(&key)
                                    .unwrap()
                                    .to_public()
                                    .into(),
                            )
                        })
                        .boxed(),
                )
                .boxed()
        }
    }

    impl Arbitrary for VotingRegistration {
        type Parameters = ();
        type Strategy = BoxedStrategy<VotingRegistration>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<([u8; 32], [u8; 32], Delegations)>(), 0..45_000_000u64)
                .prop_map(|((stake_key, rewards_addr, delegations), vp)| {
                    let stake_public_key = StakeAddress(hex::encode(stake_key));
                    let reward_address = RewardAddress(hex::encode(rewards_addr));
                    let voting_power: Value = vp.into();
                    VotingRegistration {
                        stake_public_key,
                        voting_power,
                        reward_address,
                        delegations,
                        voting_purpose: None,
                        nonce: 0,
                    }
                })
                .boxed()
        }
    }

    #[cfg(test)]
    #[test]
    fn reward_address_serde_test() {
        assert_eq!(
            serde_json::to_value(RewardAddress(
                "0x01cd3be59b212a45b99f2d26bd179c7119e2851c3b7ada415eff504683c7a5c447ebee137a684b65750e8ab5227ffb3199017bdaf069464c11".to_string()
            )).unwrap(),
            serde_json::json!("addr1q8xnhevmyy4ytwvl95nt69uuwyv79pgu8dad5s27lagydq785hzy06lwzdaxsjm9w58g4dfz0lanrxgp00d0q62xfsgsh7dfml")
        );

        assert_eq!(
            serde_json::to_value(RewardAddress(
                "0xe1b8d7b8e56a3ed89ee21bc062d284d537f843b50b68b905618b130297".to_string()
            ))
            .unwrap(),
            serde_json::json!("stake1uxud0w89dgld38hzr0qx955y65mlssa4pd5tjptp3vfs99cj39wag")
        );

        assert_eq!(
            serde_json::to_value(RewardAddress(
                "0x00cd3be59b212a45b99f2d26bd179c7119e2851c3b7ada415eff504683c7a5c447ebee137a684b65750e8ab5227ffb3199017bdaf069464c11".to_string()
            )).unwrap(),
            serde_json::json!("addr_test1qrxnhevmyy4ytwvl95nt69uuwyv79pgu8dad5s27lagydq785hzy06lwzdaxsjm9w58g4dfz0lanrxgp00d0q62xfsgs5gsfhq")
        );

        assert_eq!(
            serde_json::to_value(RewardAddress(
                "0xe0b8d7b8e56a3ed89ee21bc062d284d537f843b50b68b905618b130297".to_string()
            ))
            .unwrap(),
            serde_json::json!("stake_test1uzud0w89dgld38hzr0qx955y65mlssa4pd5tjptp3vfs99c4m0ve4")
        );
    }

    #[cfg(test)]
    #[test]
    fn parse_example() {
        assert_de_tokens(
            &Delegations::New(vec![
                (
                    Identifier::from_hex(
                        "a6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663",
                    )
                    .unwrap(),
                    3,
                ),
                (
                    Identifier::from_hex(
                        "00588e8e1d18cba576a4d35758069fe94e53f638b6faf7c07b8abd2bc5c5cdee",
                    )
                    .unwrap(),
                    1,
                ),
            ])
            .readable(),
            &[
                Token::Seq { len: None },
                Token::Seq { len: None },
                Token::Str("0xa6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663"),
                Token::U32(3),
                Token::SeqEnd,
                Token::Seq { len: None },
                Token::Str("0x00588e8e1d18cba576a4d35758069fe94e53f638b6faf7c07b8abd2bc5c5cdee"),
                Token::U32(1),
                Token::SeqEnd,
                Token::SeqEnd,
            ],
        );
    }

    #[cfg(test)]
    #[proptest]
    fn serde_json(d: Delegations) {
        assert_eq!(
            serde_json::from_str::<Delegations>(&serde_json::to_string(&d).unwrap()).unwrap(),
            d
        );
    }

    #[cfg(test)]
    #[proptest]
    fn serde_yaml(d: Delegations) {
        assert_eq!(
            serde_yaml::from_str::<Delegations>(&serde_yaml::to_string(&d).unwrap()).unwrap(),
            d
        );
    }

    #[cfg(test)]
    #[test]
    fn test_empty_delegations_are_rejected() {
        assert!(serde_json::from_str::<Delegations>(r#"[]"#,).is_err());
    }

    #[cfg(test)]
    #[test]
    fn test_u64_weight_is_rejected() {
        assert!(serde_json::from_str::<Delegations>(r#"[["0xa6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663", 4294967296]]"#,).is_err());
    }

    #[cfg(test)]
    #[test]
    fn test_legacy_delegation_is_ok() {
        assert!(serde_json::from_str::<Delegations>(
            r#""0xa6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663""#,
        )
        .is_ok());
    }

    #[cfg(test)]
    #[test]
    fn test_u32_weight_is_ok() {
        serde_json::from_str::<Delegations>(
            r#"[["0xa6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663", 4294967295]]"#,
        )
        .unwrap();
    }
}
