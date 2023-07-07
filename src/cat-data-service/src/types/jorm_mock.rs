use chain_impl_mockchain::key::AccountPublicKey;
use jormungandr_lib::interfaces::FragmentDef;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::ops::Deref;

#[serde_as]
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash)]
pub struct AccountId(#[serde_as(as = "DisplayFromStr")] pub AccountPublicKey);

impl Deref for AccountId {
    type Target = AccountPublicKey;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct VotePlanId(pub jormungandr_lib::interfaces::VotePlanId);

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Fragment(#[serde(with = "FragmentDef")] pub chain_impl_mockchain::fragment::Fragment);

impl Deref for Fragment {
    type Target = chain_impl_mockchain::fragment::Fragment;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FragmentId(
    #[serde_as(as = "DisplayFromStr")] pub chain_impl_mockchain::fragment::FragmentId,
);

impl Deref for FragmentId {
    type Target = chain_impl_mockchain::fragment::FragmentId;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct ProposalIndex(pub u8);

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Fragments {
    pub fail_fast: bool,
    pub fragments: Vec<Fragment>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum Reason {
    FragmentAlreadyInLog,
    FragmentInvalid,
}

pub const DEFAULT_POOL_NUMBER: u64 = 0;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RejectedInfo {
    pub id: FragmentId,
    pub pool_number: u64,
    pub reason: Reason,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FragmentsProcessingSummary {
    pub accepted: Vec<FragmentId>,
    pub rejected: Vec<RejectedInfo>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AccountVote {
    pub vote_plan_id: VotePlanId,
    pub votes: Vec<ProposalIndex>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::str::FromStr;

    #[test]
    fn account_id_json_test() {
        let json = json!("0000000000000000000000000000000000000000");
        let account_id: AccountId = serde_json::from_value(json).unwrap();

        assert_eq!(
            account_id,
            AccountId(
                AccountPublicKey::from_str("0000000000000000000000000000000000000000",).unwrap()
            )
        );
    }

    #[test]
    fn reason_json_test() {
        let reason = Reason::FragmentAlreadyInLog;
        let json = serde_json::to_value(&reason).unwrap();
        assert_eq!(json, json!("FragmentAlreadyInLog"));

        let reason = Reason::FragmentInvalid;
        let json = serde_json::to_value(&reason).unwrap();
        assert_eq!(json, json!("FragmentInvalid"));
    }

    #[test]
    fn rejected_info_json_test() {
        let rejected_info = RejectedInfo {
            id: FragmentId(chain_impl_mockchain::fragment::FragmentId::zero_hash()),
            pool_number: DEFAULT_POOL_NUMBER,
            reason: Reason::FragmentInvalid,
        };
        let json = serde_json::to_value(&rejected_info).unwrap();
        assert_eq!(
            json,
            json!({
                "id": "0000000000000000000000000000000000000000000000000000000000000000",
                "pool_number": 0,
                "reason": "FragmentInvalid",
            })
        );
    }

    #[test]
    fn fragments_processing_summary_json_test() {
        let summary = FragmentsProcessingSummary {
            accepted: vec![FragmentId(
                chain_impl_mockchain::fragment::FragmentId::zero_hash(),
            )],
            rejected: vec![],
        };
        let json = serde_json::to_value(&summary).unwrap();
        assert_eq!(
            json,
            json!({
                "accepted": [
                    "0000000000000000000000000000000000000000000000000000000000000000",
                ],
                "rejected": [],
            })
        );
    }

    #[test]
    fn account_vote_json_test() {
        let account_vote = AccountVote {
            vote_plan_id: VotePlanId(
                jormungandr_lib::interfaces::VotePlanId::from_hex(
                    "0000000000000000000000000000000000000000000000000000000000000000",
                )
                .unwrap(),
            ),
            votes: vec![ProposalIndex(1)],
        };
        let json = serde_json::to_value(&account_vote).unwrap();
        assert_eq!(
            json,
            json!({
                "vote_plan_id": "0000000000000000000000000000000000000000000000000000000000000000",
                "votes": [
                    1
                ]
            })
        )
    }
}
