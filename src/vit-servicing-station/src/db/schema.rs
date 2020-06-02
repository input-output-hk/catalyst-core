table! {
    chain_voteplan (vote_plan_id) {
        vote_plan_id -> Text,
        chain_vote_plan_id -> Text,
        chain_vote_starttime -> Text,
        chain_vote_endtime -> Text,
        chain_committee_endtime -> Text,
    }
}

table! {
    fund_voteplans (fund_name, vote_plan_id) {
        fund_name -> Text,
        vote_plan_id -> Text,
    }
}

table! {
    funds (fund_name) {
        fund_name -> Text,
        fund_goal -> Text,
        voting_power_info -> Text,
        rewards_info -> Text,
        fund_start_time -> Text,
        fund_end_time -> Text,
        next_fund_start_time -> Text,
    }
}

table! {
    proposals (proposal_id) {
        proposal_id -> Text,
        proposal_category -> Text,
        proposal_title -> Text,
        proposal_summary -> Text,
        proposal_problem -> Text,
        proposal_solution -> Text,
        proposal_funds -> BigInt,
        proposal_url -> Text,
        proposal_files_url -> Text,
        proposer_name -> Text,
        proposer_contact -> Text,
        proposer_url -> Text,
        chain_proposal_id -> Text,
        chain_voteplan_id -> Text,
        chain_proposal_index -> BigInt,
        chain_vote_start_time -> BigInt,
        chain_vote_end_time -> BigInt,
        chain_committee_end_time -> BigInt,
        chain_vote_options -> Text,
    }
}

table! {
    voteplan (vote_plan_id) {
        vote_plan_id -> Nullable<Text>,
        chain_vote_plan_id -> Text,
        chain_vote_starttime -> Text,
        chain_vote_endtime -> Text,
        chain_committee_endtime -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    chain_voteplan,
    fund_voteplans,
    funds,
    proposals,
    voteplan,
);
