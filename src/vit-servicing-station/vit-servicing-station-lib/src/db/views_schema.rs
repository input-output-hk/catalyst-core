use diesel::table;

table! {
    full_proposals_info {
        id -> Int4,
        proposal_id -> Varchar,
        proposal_category -> Varchar,
        proposal_title -> Varchar,
        proposal_summary -> Varchar,
        proposal_public_key -> Varchar,
        proposal_funds -> Int8,
        proposal_url -> Varchar,
        proposal_files_url -> Varchar,
        proposal_impact_score -> Int8,
        proposer_name -> Varchar,
        proposer_contact -> Varchar,
        proposer_url -> Varchar,
        proposer_relevant_experience -> Varchar,
        chain_proposal_id -> Bytea,
        chain_vote_options -> Varchar,
        challenge_id -> Int4,

        reviews_count -> Int4,

        chain_vote_start_time -> Int8,
        chain_vote_end_time -> Int8,
        chain_committee_end_time -> Int8,
        chain_voteplan_payload -> Varchar,
        chain_vote_encryption_key -> Varchar,
        fund_id -> Int4,

        challenge_type -> Varchar,
        proposal_solution -> Nullable<Varchar>,
        proposal_brief -> Nullable<Varchar>,
        proposal_importance -> Nullable<Varchar>,
        proposal_goal -> Nullable<Varchar>,
        proposal_metrics -> Nullable<Varchar>,

        chain_proposal_index -> Int8,
        chain_voteplan_id -> Varchar,

        group_id -> Varchar,
    }
}
