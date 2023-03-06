use serde::Serialize;

#[derive(Serialize, Clone, Default)]
pub struct VotePlan {
    id: i32,
    chain_voteplan_id: String,
    chain_vote_start_time: String,
    chain_vote_end_time: String,
    chain_committee_end_time: String,
    chain_voteplan_payload: String,
    fund_id: i32,
    voting_token: String,
}

#[derive(Serialize, Clone, Default)]
pub struct VoterGroup {
    id: String,
    voting_token: String,
}

#[derive(Serialize, Clone, Default)]
pub struct Highlights {
    sponsor: String,
}

#[derive(Serialize, Clone, Default)]
pub struct Challenge {
    id: i32,
    challenge_type: String,
    title: String,
    description: String,
    rewards_total: i64,
    fund_id: i32,
    challenge_url: String,
    highlights: Highlights,
}

#[derive(Serialize, Clone, Default)]
pub struct Goal {
    id: i32,
    goal_name: String,
    fund_id: i32,
}

#[derive(Serialize, Clone, Default)]
pub struct Fund {
    id: i32,
    fund_name: String,
    fund_goal: String,
    voting_power_info: String,
    voting_power_threshold: i64,
    rewards_info: String,
    fund_start_time: String,
    fund_end_time: String,
    next_fund_start_time: String,
    registration_snapshot_time: String,
    next_registration_snapshot_time: String,
    chain_vote_plans: Vec<VotePlan>,
    groups: Vec<VoterGroup>,
    challenges: Vec<Challenge>,
    goals: Vec<Goal>,
    insight_sharing_start: String,
    proposal_submission_start: String,
    refine_proposals_start: String,
    finalize_proposals_start: String,
    proposal_assessment_start: String,
    assessment_qa_start: String,
    snapshot_start: String,
    voting_start: String,
    voting_end: String,
    tallying_end: String,
}

#[derive(Serialize, Clone, Default)]
pub struct FundIDs(Vec<i32>);

pub trait FundDb {
    fn get_current_fund(&self) -> Fund;
    fn get_fund_by_id(&self, id: i32) -> Fund;
    fn get_fund_ids(&self) -> FundIDs;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vote_plan_json_test() {
        let vote_plan = VotePlan {
            id: 0,
            chain_voteplan_id: "chain_voteplan_id".to_string(),
            chain_vote_start_time: "chain_vote_start_time".to_string(),
            chain_vote_end_time: "chain_vote_end_time".to_string(),
            chain_committee_end_time: "chain_committee_end_time".to_string(),
            chain_voteplan_payload: "chain_voteplan_payload".to_string(),
            fund_id: 0,
            voting_token: "voting_token".to_string(),
        };
        let json = serde_json::to_string(&vote_plan).unwrap();
        assert_eq!(
            json,
            r#"{"id":0,"chain_voteplan_id":"chain_voteplan_id","chain_vote_start_time":"chain_vote_start_time","chain_vote_end_time":"chain_vote_end_time","chain_committee_end_time":"chain_committee_end_time","chain_voteplan_payload":"chain_voteplan_payload","fund_id":0,"voting_token":"voting_token"}"#
        );
    }

    #[test]
    fn voter_group_json_test() {
        let voter_group = VoterGroup {
            id: "id".to_string(),
            voting_token: "voting_token".to_string(),
        };
        let json = serde_json::to_string(&voter_group).unwrap();
        assert_eq!(json, r#"{"id":"id","voting_token":"voting_token"}"#);
    }

    #[test]
    fn challenge_json_test() {
        let challenge = Challenge {
            id: 0,
            challenge_type: "challenge_type".to_string(),
            title: "title".to_string(),
            description: "description".to_string(),
            rewards_total: 0,
            fund_id: 0,
            challenge_url: "challenge_url".to_string(),
            highlights: Highlights {
                sponsor: "sponsor".to_string(),
            },
        };
        let json = serde_json::to_string(&challenge).unwrap();
        assert_eq!(
            json,
            r#"{"id":0,"challenge_type":"challenge_type","title":"title","description":"description","rewards_total":0,"fund_id":0,"challenge_url":"challenge_url","highlights":{"sponsor":"sponsor"}}"#
        );
    }

    #[test]
    fn goal_json_test() {
        let goal = Goal {
            id: 0,
            goal_name: "goal_name".to_string(),
            fund_id: 0,
        };
        let json = serde_json::to_string(&goal).unwrap();
        assert_eq!(json, r#"{"id":0,"goal_name":"goal_name","fund_id":0}"#);
    }

    #[test]
    fn fund_json_test() {
        let fund = Fund {
            id: 0,
            fund_name: "fund_name".to_string(),
            fund_goal: "fund_goal".to_string(),
            voting_power_info: "voting_power_info".to_string(),
            voting_power_threshold: 0,
            rewards_info: "rewards_info".to_string(),
            fund_start_time: "fund_start_time".to_string(),
            fund_end_time: "fund_end_time".to_string(),
            next_fund_start_time: "next_fund_start_time".to_string(),
            registration_snapshot_time: "registration_snapshot_time".to_string(),
            next_registration_snapshot_time: "next_registration_snapshot_time".to_string(),
            chain_vote_plans: vec![],
            groups: vec![],
            challenges: vec![],
            goals: vec![],
            insight_sharing_start: "insight_sharing_start".to_string(),
            proposal_submission_start: "proposal_submission_start".to_string(),
            refine_proposals_start: "refine_proposals_start".to_string(),
            finalize_proposals_start: "finalize_proposals_start".to_string(),
            proposal_assessment_start: "proposal_assessment_start".to_string(),
            assessment_qa_start: "assessment_qa_start".to_string(),
            snapshot_start: "snapshot_start".to_string(),
            voting_start: "voting_start".to_string(),
            voting_end: "voting_end".to_string(),
            tallying_end: "tallying_end".to_string(),
        };
        let json = serde_json::to_string(&fund).unwrap();
        assert_eq!(
            json,
            r#"{"id":0,"fund_name":"fund_name","fund_goal":"fund_goal","voting_power_info":"voting_power_info","voting_power_threshold":0,"rewards_info":"rewards_info","fund_start_time":"fund_start_time","fund_end_time":"fund_end_time","next_fund_start_time":"next_fund_start_time","registration_snapshot_time":"registration_snapshot_time","next_registration_snapshot_time":"next_registration_snapshot_time","chain_vote_plans":[],"groups":[],"challenges":[],"goals":[],"insight_sharing_start":"insight_sharing_start","proposal_submission_start":"proposal_submission_start","refine_proposals_start":"refine_proposals_start","finalize_proposals_start":"finalize_proposals_start","proposal_assessment_start":"proposal_assessment_start","assessment_qa_start":"assessment_qa_start","snapshot_start":"snapshot_start","voting_start":"voting_start","voting_end":"voting_end","tallying_end":"tallying_end"}"#
        );
    }

    #[test]
    fn fund_ids_json_test() {
        let fund_ids = FundIDs(vec![0, 1]);
        let json = serde_json::to_string(&fund_ids).unwrap();
        assert_eq!(json, r#"[0,1]"#);
    }
}
