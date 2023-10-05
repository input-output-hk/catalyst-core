use jortestkit::csv::CsvFileBuilder;
use std::path::Path;
use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use vit_servicing_station_lib_f10::db::models::community_advisors_reviews::AdvisorReview;
use vit_servicing_station_lib_f10::db::models::goals::InsertGoal;
use vit_servicing_station_lib_f10::db::models::proposals::{
    FullProposalInfo, ProposalChallengeInfo,
};
use vit_servicing_station_lib_f10::{
    db::models::{challenges::Challenge, funds::Fund, voteplans::Voteplan},
    utils::datetime::unix_timestamp_to_datetime,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Cannot format csv file with funds due to : {0}")]
    CannotBuildCsvWithFunds(String),
}

pub struct CsvConverter;

impl CsvConverter {
    pub fn funds<P: AsRef<Path>>(&self, funds: Vec<Fund>, path: P) -> Result<(), Error> {
        let headers = vec![
            "id",
            "fund_name",
            "voting_power_threshold",
            "fund_goal",
            "fund_start_time",
            "fund_end_time",
            "next_fund_start_time",
            "registration_snapshot_time",
            "next_registration_snapshot_time",
            "insight_sharing_start",
            "proposal_submission_start",
            "refine_proposals_start",
            "finalize_proposals_start",
            "proposal_assessment_start",
            "assessment_qa_start",
            "snapshot_start",
            "voting_start",
            "voting_end",
            "tallying_end",
            "results_url",
            "survey_url",
        ];
        let content: Vec<Vec<String>> = funds.iter().map(convert_fund).collect();
        self.build_file(headers, content, path)
    }

    pub fn voteplans<P: AsRef<Path>>(
        &self,
        voteplans: Vec<Voteplan>,
        path: P,
    ) -> Result<(), Error> {
        let headers = vec![
            "id",
            "chain_voteplan_id",
            "chain_vote_start_time",
            "chain_vote_end_time",
            "chain_committee_end_time",
            "chain_voteplan_payload",
            "chain_vote_encryption_key",
            "fund_id",
        ];
        let content: Vec<Vec<String>> = voteplans.iter().map(convert_voteplan).collect();
        self.build_file(headers, content, path)
    }

    pub fn proposals<P: AsRef<Path>>(
        &self,
        proposals: Vec<FullProposalInfo>,
        path: P,
    ) -> Result<(), Error> {
        let headers = vec![
            "internal_id",
            "category_name",
            "proposal_id",
            "proposal_title",
            "proposal_summary",
            "proposal_url",
            "proposal_files_url",
            "proposal_public_key",
            "proposal_funds",
            "proposal_impact_score",
            "proposer_email",
            "proposer_name",
            "proposer_url",
            "proposer_relevant_experience",
            "chain_proposal_id",
            "chain_proposal_index",
            "chain_vote_options",
            "chain_vote_type",
            "chain_vote_action",
            "id",
            "chain_voteplan_id",
            "chain_vote_start_time",
            "chain_vote_end_time",
            "chain_committe",
            "challenge_id",
            "proposal_solution",
            "proposal_brief",
            "proposal_importance",
            "proposal_goal",
            "proposal_metrics",
        ];

        let content: Vec<Vec<String>> = proposals.iter().map(convert_proposal).collect();
        self.build_file(headers, content, path)
    }

    pub fn challenges<P: AsRef<Path>>(
        &self,
        challenges: Vec<Challenge>,
        path: P,
    ) -> Result<(), Error> {
        let headers = vec![
            "id",
            "challenge_type",
            "title",
            "description",
            "rewards_total",
            "proposers_rewards",
            "fund_id",
            "challenge_url",
        ];

        let content: Vec<Vec<String>> = challenges.iter().map(convert_challenge).collect();
        self.build_file(headers, content, path)
    }

    pub fn advisor_reviews<P: AsRef<Path>>(
        &self,
        challenges: Vec<AdvisorReview>,
        path: P,
    ) -> Result<(), Error> {
        let headers = vec![
            "id",
            "proposal_id",
            "assessor",
            "impact_alignment_rating_given",
            "impact_alignment_note",
            "feasibility_rating_given",
            "feasibility_note",
            "auditability_rating_given",
            "auditability_note",
            "excellent",
            "good",
        ];

        let content: Vec<Vec<String>> = challenges.iter().map(convert_advisor_review).collect();
        self.build_file(headers, content, path)
    }

    pub fn goals<P: AsRef<Path>>(&self, goals: Vec<InsertGoal>, path: P) -> Result<(), Error> {
        let headers = vec!["goal_name", "fund_id"];

        let content: Vec<Vec<String>> = goals.iter().map(convert_goals).collect();
        self.build_file(headers, content, path)
    }

    fn build_file<P: AsRef<Path>>(
        &self,
        headers: Vec<&str>,
        content: Vec<Vec<String>>,
        path: P,
    ) -> Result<(), Error> {
        let mut csv_loader: CsvFileBuilder = CsvFileBuilder::from_path(path);
        csv_loader
            .with_header(headers)
            .with_contents(content)
            .build()
            .map_err(|e| Error::CannotBuildCsvWithFunds(e.to_string()))
    }
}

fn convert_proposal(proposal: &FullProposalInfo) -> Vec<String> {
    let (solution, brief, importance, goal, metrics) = match &proposal.challenge_info {
        ProposalChallengeInfo::Simple(data) => (
            data.proposal_solution.clone(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        ),
        ProposalChallengeInfo::CommunityChoice(data) => (
            "".to_string(),
            data.proposal_brief.clone(),
            data.proposal_importance.clone(),
            data.proposal_goal.clone(),
            data.proposal_metrics.clone(),
        ),
    };
    let proposal = &proposal.proposal;

    vec![
        proposal.internal_id.to_string(),
        proposal.proposal_category.category_name.to_string(),
        proposal.proposal_id.to_string(),
        proposal.proposal_title.to_string(),
        proposal.proposal_summary.to_string(),
        proposal.proposal_url.to_string(),
        proposal.proposal_files_url.to_string(),
        proposal.proposal_public_key.to_string(),
        proposal.proposal_funds.to_string(),
        proposal.proposal_impact_score.to_string(),
        proposal.proposer.proposer_email.to_string(),
        proposal.proposer.proposer_name.to_string(),
        proposal.proposer.proposer_url.to_string(),
        proposal.proposer.proposer_relevant_experience.to_string(),
        std::str::from_utf8(&proposal.chain_proposal_id)
            .unwrap()
            .to_string(),
        proposal.chain_proposal_index.to_string(),
        proposal.chain_vote_options.as_csv_string(),
        proposal.chain_voteplan_payload.to_string(),
        "off_chain".to_string(),
        proposal.proposal_id.to_string(),
        proposal.chain_voteplan_id.to_string(),
        unix_timestamp_to_rfc3339(proposal.chain_vote_start_time),
        unix_timestamp_to_rfc3339(proposal.chain_vote_end_time),
        unix_timestamp_to_rfc3339(proposal.chain_committee_end_time),
        proposal.challenge_id.to_string(),
        solution,
        brief,
        importance,
        goal,
        metrics,
    ]
}

fn convert_fund(fund: &Fund) -> Vec<String> {
    // destructure the object to get a compile-time exhaustivity check, even if we already have
    // tests for this, it's easier to keep it up-to-date
    let Fund {
        id,
        fund_name,
        fund_goal,
        voting_power_threshold,
        fund_start_time,
        fund_end_time,
        next_fund_start_time,
        registration_snapshot_time,
        next_registration_snapshot_time,
        chain_vote_plans: _,
        challenges: _,
        stage_dates,
        goals: _,
        results_url,
        survey_url,
    } = fund;

    // TODO: can we leverage serde to build these vectors?
    vec![
        id.to_string(),
        fund_name.to_string(),
        voting_power_threshold.to_string(),
        fund_goal.to_string(),
        unix_timestamp_to_rfc3339(*fund_start_time),
        unix_timestamp_to_rfc3339(*fund_end_time),
        unix_timestamp_to_rfc3339(*next_fund_start_time),
        unix_timestamp_to_rfc3339(*registration_snapshot_time),
        unix_timestamp_to_rfc3339(*next_registration_snapshot_time),
        unix_timestamp_to_rfc3339(stage_dates.insight_sharing_start),
        unix_timestamp_to_rfc3339(stage_dates.proposal_submission_start),
        unix_timestamp_to_rfc3339(stage_dates.refine_proposals_start),
        unix_timestamp_to_rfc3339(stage_dates.finalize_proposals_start),
        unix_timestamp_to_rfc3339(stage_dates.proposal_assessment_start),
        unix_timestamp_to_rfc3339(stage_dates.assessment_qa_start),
        unix_timestamp_to_rfc3339(stage_dates.snapshot_start),
        unix_timestamp_to_rfc3339(stage_dates.voting_start),
        unix_timestamp_to_rfc3339(stage_dates.voting_end),
        unix_timestamp_to_rfc3339(stage_dates.tallying_end),
        results_url.to_string(),
        survey_url.to_string(),
    ]
}

fn convert_voteplan(voteplan: &Voteplan) -> Vec<String> {
    vec![
        voteplan.id.to_string(),
        voteplan.chain_voteplan_id.to_string(),
        unix_timestamp_to_rfc3339(voteplan.chain_vote_start_time),
        unix_timestamp_to_rfc3339(voteplan.chain_vote_end_time),
        unix_timestamp_to_rfc3339(voteplan.chain_committee_end_time),
        voteplan.chain_voteplan_payload.to_string(),
        voteplan.chain_vote_encryption_key.to_string(),
        voteplan.fund_id.to_string(),
    ]
}

fn convert_challenge(challenge: &Challenge) -> Vec<String> {
    vec![
        challenge.id.to_string(),
        challenge.challenge_type.to_string(),
        challenge.title.clone(),
        challenge.description.clone(),
        challenge.rewards_total.to_string(),
        challenge.proposers_rewards.to_string(),
        challenge.fund_id.to_string(),
        challenge.challenge_url.clone(),
    ]
}

fn convert_advisor_review(review: &AdvisorReview) -> Vec<String> {
    vec![
        review.id.to_string(),
        review.proposal_id.to_string(),
        review.assessor.to_string(),
        review.impact_alignment_rating_given.to_string(),
        review.impact_alignment_note.to_string(),
        review.feasibility_rating_given.to_string(),
        review.feasibility_note.to_string(),
        review.auditability_rating_given.to_string(),
        review.auditability_note.to_string(),
        (review.ranking as u8 == 0).to_string(),
        (review.ranking as u8 == 1).to_string(),
    ]
}

fn convert_goals(goal: &InsertGoal) -> Vec<String> {
    let InsertGoal { goal_name, fund_id } = goal;
    vec![goal_name.to_string(), fund_id.to_string()]
}

fn unix_timestamp_to_rfc3339(timestamp: i64) -> String {
    unix_timestamp_to_datetime(timestamp)
        .format(&Rfc3339)
        .unwrap()
}
