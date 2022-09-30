use crate::config::{SnapshotError, SnapshotInitials};
use hersir::builder::Wallet as WalletSettings;
use jormungandr_lib::crypto::account::Identifier;
use proptest::{
    arbitrary::Arbitrary, prelude::*, strategy::BoxedStrategy, test_runner::TestRunner,
};
use snapshot_lib::VoterHIR;
use std::collections::BTreeMap;
use thor::WalletAlias;

// TODO: this is a temporary impl until the snapshot service is available as a standalone
// microservice.
#[derive(Debug, Default)]
pub struct VoterSnapshot {
    hirs_by_tag: BTreeMap<String, Vec<VoterHIR>>,
}

impl VoterSnapshot {
    pub fn from_config_or_default(
        defined_wallets: Vec<(WalletAlias, &WalletSettings)>,
        snapshot_config: &Option<SnapshotInitials>,
    ) -> Result<Self, SnapshotError> {
        if let Some(snapshot_config) = snapshot_config {
            let mut snapshot = Self::default();
            snapshot.update_tag(
                snapshot_config.tag.clone(),
                snapshot_config.as_voters_hirs(defined_wallets)?,
            );
            Ok(snapshot)
        } else {
            Ok(Self::dummy())
        }
    }
    pub fn get_voting_power(&self, tag: &str, voting_key: &Identifier) -> Vec<VoterHIR> {
        self.hirs_by_tag
            .get(tag)
            .iter()
            .flat_map(|hirs| hirs.iter())
            .filter(|voter| &voter.voting_key == voting_key)
            .cloned()
            .collect()
    }

    pub fn update_tag(&mut self, tag: String, voter_hirs: Vec<VoterHIR>) {
        self.hirs_by_tag.insert(tag, voter_hirs);
    }

    pub fn tags(&self) -> Vec<String> {
        self.hirs_by_tag.keys().cloned().collect()
    }

    pub fn get_snapshot(&self, tag: &str) -> Vec<VoterHIR> {
        self.hirs_by_tag.get(tag).cloned().unwrap_or_default()
    }

    pub fn dummy() -> Self {
        let mut test_runner = TestRunner::deterministic();
        Self::arbitrary_with(())
            .new_tree(&mut test_runner)
            .unwrap()
            .current()
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

impl Arbitrary for VoterSnapshot {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        let tags = vec![
            String::from("latest"),
            String::from("fund8"),
            String::from("nightly"),
        ];
        any_with::<(Vec<ArbitraryVoterHIR>, Vec<ArbitraryVoterHIR>)>((
            (Default::default(), Some("direct".to_string())),
            (Default::default(), Some("dreps".to_string())),
        ))
        .prop_map(move |(dreps, voters)| {
            let mut hirs_by_tag = BTreeMap::new();
            let hirs = dreps
                .into_iter()
                .map(|x| x.0)
                .chain(voters.into_iter().map(|x| x.0))
                .collect::<Vec<_>>();
            for tag in tags.clone() {
                hirs_by_tag.insert(tag, hirs.clone());
            }
            Self { hirs_by_tag }
        })
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_voting_power() {
        let mut hirs = BTreeMap::new();
        hirs.insert("a".to_string(), Vec::new());
        hirs.insert("b".to_string(), Vec::new());
        hirs.insert("c".to_string(), Vec::new());

        let key = [0u8; 32];
        let vk = Identifier::from_hex(&hex::encode(key)).unwrap();

        let mut snapshot = VoterSnapshot { hirs_by_tag: hirs };
        assert_eq!(snapshot.get_voting_power("a", &vk), Vec::new());
        let entries = vec![
            VoterHIR {
                voting_key: vk.clone(),
                voting_power: 1.into(),
                voting_group: "g".to_string(),
            },
            VoterHIR {
                voting_key: vk.clone(),
                voting_power: 1.into(),
                voting_group: "gg".to_string(),
            },
        ];
        snapshot.update_tag("a".to_string(), entries.clone());
        assert_eq!(snapshot.get_voting_power("a", &vk), entries);
    }

    #[test]
    fn test_tags() {
        let mut hirs = BTreeMap::new();
        hirs.insert("a".to_string(), Vec::new());
        hirs.insert("b".to_string(), Vec::new());
        hirs.insert("c".to_string(), Vec::new());
        assert_eq!(
            &[String::from("a"), String::from("b"), String::from("c")],
            VoterSnapshot { hirs_by_tag: hirs }.tags().as_slice()
        );
    }
}
