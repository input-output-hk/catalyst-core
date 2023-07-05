use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Fragment(pub String);

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FragmentId(pub String);

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Fragments {
    pub fail_fast: bool,
    pub fragments: Vec<Fragment>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum Reason {
    FragmentAlreadyInLog,
    FragmentInvalid,
    PreviousFragmentInvalid,
    PoolOverflow,
}

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
    pub vote_plan_id: String,
    pub votes: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn fragmenst_json_test() {
        let json = json!({
            "fail_fast": false,
            "fragments": [
                "a50a80e0ce6cb8e19d4381dc2a521c1d3ab8a532029131e440548625b2a4d3e8",
            ]
        });

        let fragments: Fragments = serde_json::from_value(json).unwrap();
        assert_eq!(
            fragments,
            Fragments {
                fail_fast: false,
                fragments: vec![Fragment(
                    "a50a80e0ce6cb8e19d4381dc2a521c1d3ab8a532029131e440548625b2a4d3e8".to_string()
                ),],
            }
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

        let reason = Reason::PreviousFragmentInvalid;
        let json = serde_json::to_value(&reason).unwrap();
        assert_eq!(json, json!("PreviousFragmentInvalid"));

        let reason = Reason::PoolOverflow;
        let json = serde_json::to_value(&reason).unwrap();
        assert_eq!(json, json!("PoolOverflow"));
    }

    #[test]
    fn rejected_info_json_test() {
        let rejected_info = RejectedInfo {
            id: FragmentId(
                "a50a80e0ce6cb8e19d4381dc2a521c1d3ab8a532029131e440548625b2a4d3e8".to_string(),
            ),
            pool_number: 0,
            reason: Reason::FragmentInvalid,
        };
        let json = serde_json::to_value(&rejected_info).unwrap();
        assert_eq!(
            json,
            json!({
                "id": "a50a80e0ce6cb8e19d4381dc2a521c1d3ab8a532029131e440548625b2a4d3e8",
                "pool_number": 0,
                "reason": "FragmentInvalid",
            })
        );
    }

    #[test]
    fn fragments_processing_summary_json_test() {
        let summary = FragmentsProcessingSummary {
            accepted: vec![FragmentId(
                "a50a80e0ce6cb8e19d4381dc2a521c1d3ab8a532029131e440548625b2a4d3e8".to_string(),
            )],
            rejected: vec![],
        };
        let json = serde_json::to_value(&summary).unwrap();
        assert_eq!(
            json,
            json!({
                "accepted": [
                    "a50a80e0ce6cb8e19d4381dc2a521c1d3ab8a532029131e440548625b2a4d3e8",
                ],
                "rejected": [],
            })
        );
    }

    #[test]
    fn account_vote_json_test() {
        let account_vote = AccountVote {
            vote_plan_id: "a50a80e0ce6cb8e19d4381dc2a521c1d3ab8a532029131e440548625b2a4d3e8"
                .to_string(),
            votes: vec![1],
        };
        let json = serde_json::to_value(&account_vote).unwrap();
        assert_eq!(
            json,
            json!({
                "vote_plan_id": "a50a80e0ce6cb8e19d4381dc2a521c1d3ab8a532029131e440548625b2a4d3e8",
                "votes": [
                    1
                ]
            })
        )
    }
}
