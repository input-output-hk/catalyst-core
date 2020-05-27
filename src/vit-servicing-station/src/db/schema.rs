table! {
    proposals (id) {
        id -> Integer,
        proposal_category -> Text,
        proposal_id -> Text,
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
