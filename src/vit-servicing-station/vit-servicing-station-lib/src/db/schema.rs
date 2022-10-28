// @generated automatically by Diesel CLI.

diesel::table! {
    api_tokens (token) {
        token -> Bytea,
        creation_time -> Int8,
        expire_time -> Int8,
    }
}

diesel::table! {
    challenges (internal_id) {
        internal_id -> Int4,
        id -> Int4,
        challenge_type -> Varchar,
        title -> Varchar,
        description -> Varchar,
        rewards_total -> Int8,
        proposers_rewards -> Int8,
        fund_id -> Int4,
        challenge_url -> Varchar,
        highlights -> Nullable<Varchar>,
    }
}

diesel::table! {
    community_advisors_reviews (id) {
        id -> Int4,
        proposal_id -> Int4,
        assessor -> Varchar,
        impact_alignment_rating_given -> Int4,
        impact_alignment_note -> Varchar,
        feasibility_rating_given -> Int4,
        feasibility_note -> Varchar,
        auditability_rating_given -> Int4,
        auditability_note -> Varchar,
        ranking -> Int4,
    }
}

diesel::table! {
    contributions (stake_public_key, voting_key, voting_group, snapshot_tag) {
        stake_public_key -> Text,
        reward_address -> Text,
        value -> Int8,
        voting_key -> Text,
        voting_group -> Text,
        snapshot_tag -> Text,
    }
}

diesel::table! {
    funds (id) {
        id -> Int4,
        fund_name -> Varchar,
        fund_goal -> Varchar,
        registration_snapshot_time -> Int8,
        next_registration_snapshot_time -> Int8,
        voting_power_threshold -> Int8,
        fund_start_time -> Int8,
        fund_end_time -> Int8,
        next_fund_start_time -> Int8,
        insight_sharing_start -> Int8,
        proposal_submission_start -> Int8,
        refine_proposals_start -> Int8,
        finalize_proposals_start -> Int8,
        proposal_assessment_start -> Int8,
        assessment_qa_start -> Int8,
        snapshot_start -> Int8,
        voting_start -> Int8,
        voting_end -> Int8,
        tallying_end -> Int8,
        results_url -> Varchar,
        survey_url -> Varchar,
    }
}

diesel::table! {
    goals (id) {
        id -> Int4,
        goal_name -> Varchar,
        fund_id -> Int4,
    }
}

diesel::table! {
    groups (token_identifier, fund_id) {
        fund_id -> Int4,
        token_identifier -> Varchar,
        group_id -> Varchar,
    }
}

diesel::table! {
    proposal_community_choice_challenge (proposal_id) {
        proposal_id -> Varchar,
        proposal_brief -> Nullable<Varchar>,
        proposal_importance -> Nullable<Varchar>,
        proposal_goal -> Nullable<Varchar>,
        proposal_metrics -> Nullable<Varchar>,
    }
}

diesel::table! {
    proposal_simple_challenge (proposal_id) {
        proposal_id -> Varchar,
        proposal_solution -> Nullable<Varchar>,
    }
}

diesel::table! {
    proposals (id) {
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
    }
}

diesel::table! {
    proposals_voteplans (id) {
        id -> Int4,
        proposal_id -> Varchar,
        chain_voteplan_id -> Varchar,
        chain_proposal_index -> Int8,
    }
}

diesel::table! {
    snapshots (tag) {
        tag -> Text,
        last_updated -> Int8,
    }
}

diesel::table! {
    voteplans (id) {
        id -> Int4,
        chain_voteplan_id -> Varchar,
        chain_vote_start_time -> Int8,
        chain_vote_end_time -> Int8,
        chain_committee_end_time -> Int8,
        chain_voteplan_payload -> Varchar,
        chain_vote_encryption_key -> Varchar,
        fund_id -> Int4,
        token_identifier -> Varchar,
    }
}

diesel::table! {
    voters (voting_key, voting_group, snapshot_tag) {
        voting_key -> Text,
        voting_power -> Int8,
        voting_group -> Text,
        snapshot_tag -> Text,
    }
}

diesel::table! {
    votes (fragment_id) {
        fragment_id -> Text,
        caster -> Text,
        proposal -> Int4,
        voteplan_id -> Text,
        time -> Float4,
        choice -> Nullable<Int2>,
        raw_fragment -> Text,
    }
}

diesel::joinable!(contributions -> snapshots (snapshot_tag));
diesel::joinable!(goals -> funds (fund_id));
diesel::joinable!(voters -> snapshots (snapshot_tag));

diesel::allow_tables_to_appear_in_same_query!(
    api_tokens,
    challenges,
    community_advisors_reviews,
    contributions,
    funds,
    goals,
    groups,
    proposal_community_choice_challenge,
    proposal_simple_challenge,
    proposals,
    proposals_voteplans,
    snapshots,
    voteplans,
    voters,
    votes,
);
