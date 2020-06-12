table! {
    api_tokens (token) {
        token -> Text,
        creation_time -> Text,
        expire_time -> Text,
    }
}

table! {
    funds (id) {
        id -> Integer,
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
    proposals (id) {
        id -> Integer,
        proposal_id -> Text,
        proposal_category -> Text,
        proposal_title -> Text,
        proposal_summary -> Text,
        proposal_problem -> Text,
        proposal_solution -> Text,
        proposal_public_key -> Text,
        proposal_funds -> BigInt,
        proposal_url -> Text,
        proposal_files_url -> Text,
        proposer_name -> Text,
        proposer_contact -> Text,
        proposer_url -> Text,
        chain_proposal_id -> Binary,
        chain_proposal_index -> BigInt,
        chain_vote_options -> Text,
        chain_voteplan_id -> Text,
    }
}

table! {
    voteplans (id) {
        id -> Integer,
        chain_voteplan_id -> Text,
        chain_vote_start_time -> Text,
        chain_vote_end_time -> Text,
        chain_committee_end_time -> Text,
        chain_voteplan_payload -> Text,
        fund_id -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(api_tokens, funds, proposals, voteplans,);
