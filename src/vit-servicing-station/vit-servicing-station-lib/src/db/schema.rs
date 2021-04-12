table! {
    api_tokens (token) {
        token -> Binary,
        creation_time -> BigInt,
        expire_time -> BigInt,
    }
}

table! {
    challenges (id) {
        id -> Integer,
        challenge_type -> Text,
        title -> Text,
        description -> Text,
        rewards_total -> BigInt,
        proposers_rewards -> BigInt,
        fund_id -> Integer,
        challenge_url -> Text,
    }
}

table! {
    funds (id) {
        id -> Integer,
        fund_name -> Text,
        fund_goal -> Text,
        voting_power_info -> Text,
        voting_power_threshold -> BigInt,
        rewards_info -> Text,
        fund_start_time -> BigInt,
        fund_end_time -> BigInt,
        next_fund_start_time -> BigInt,
    }
}

table! {
    proposal_community_choice_challenge (proposal_id) {
        proposal_id -> Text,
        proposal_brief -> Nullable<Text>,
        proposal_importance -> Nullable<Text>,
        proposal_goal -> Nullable<Text>,
        proposal_metrics -> Nullable<Text>,
    }
}

table! {
    proposal_simple_challenge (proposal_id) {
        proposal_id -> Text,
        proposal_solution -> Nullable<Text>,
    }
}

table! {
    proposals (id) {
        id -> Integer,
        proposal_id -> Text,
        proposal_category -> Text,
        proposal_title -> Text,
        proposal_summary -> Text,
        proposal_public_key -> Text,
        proposal_funds -> BigInt,
        proposal_url -> Text,
        proposal_files_url -> Text,
        proposal_impact_score -> BigInt,
        proposer_name -> Text,
        proposer_contact -> Text,
        proposer_url -> Text,
        proposer_relevant_experience -> Text,
        chain_proposal_id -> Binary,
        chain_proposal_index -> BigInt,
        chain_vote_options -> Text,
        chain_voteplan_id -> Text,
        challenge_id -> Integer,
    }
}

table! {
    voteplans (id) {
        id -> Integer,
        chain_voteplan_id -> Text,
        chain_vote_start_time -> BigInt,
        chain_vote_end_time -> BigInt,
        chain_committee_end_time -> BigInt,
        chain_voteplan_payload -> Text,
        chain_vote_encryption_key -> Text,
        fund_id -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(
    api_tokens,
    challenges,
    funds,
    proposal_community_choice_challenge,
    proposal_simple_challenge,
    proposals,
    voteplans,
);
