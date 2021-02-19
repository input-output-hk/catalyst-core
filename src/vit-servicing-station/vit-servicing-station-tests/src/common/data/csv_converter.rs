use jortestkit::csv::CsvFileBuilder;
use std::path::Path;
use thiserror::Error;
use vit_servicing_station_lib::db::models::proposals::{FullProposalInfo, ProposalChallengeInfo};
use vit_servicing_station_lib::{
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
            "voting_power_info",
            "rewards_info",
            "fund_start_time",
            "fund_end_time",
            "next_fund_start_time",
        ];
        let content: Vec<Vec<String>> = funds.iter().map(|x| convert_fund(x)).collect();
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
        let content: Vec<Vec<String>> = voteplans.iter().map(|x| convert_voteplan(x)).collect();
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

        let content: Vec<Vec<String>> = proposals.iter().map(|x| convert_proposal(x)).collect();
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
            "fund_id",
            "challenge_url",
        ];

        let content: Vec<Vec<String>> = challenges.iter().map(|x| convert_challenge(x)).collect();
        self.build_file(headers, content, path)
    }

    fn build_file<P: AsRef<Path>>(
        &self,
        headers: Vec<&str>,
        content: Vec<Vec<String>>,
        path: P,
    ) -> Result<(), Error> {
        let mut csv_loader: CsvFileBuilder = CsvFileBuilder::from_path(path);
        Ok(csv_loader
            .with_header(headers)
            .with_contents(content)
            .build()
            .map_err(|e| Error::CannotBuildCsvWithFunds(e.to_string()))?)
    }
}

fn convert_proposal(proposal: &FullProposalInfo) -> Vec<String> {
    let (solution, brief, importance, goal, metrics) =
        match &proposal.proposal_challenge_specific_data {
            ProposalChallengeInfo::Simple(data) => (
                data.proposal_solution.clone(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ),
            ProposalChallengeInfo::Community(data) => (
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
        unix_timestamp_to_datetime(proposal.chain_vote_start_time).to_rfc3339(),
        unix_timestamp_to_datetime(proposal.chain_vote_end_time).to_rfc3339(),
        unix_timestamp_to_datetime(proposal.chain_committee_end_time).to_rfc3339(),
        proposal.challenge_id.to_string(),
        solution,
        brief,
        importance,
        goal,
        metrics,
    ]
}

fn convert_fund(fund: &Fund) -> Vec<String> {
    vec![
        fund.id.to_string(),
        fund.fund_name.to_string(),
        fund.voting_power_threshold.to_string(),
        fund.fund_goal.to_string(),
        fund.voting_power_info.to_string(),
        fund.rewards_info.to_string(),
        unix_timestamp_to_datetime(fund.fund_start_time).to_rfc3339(),
        unix_timestamp_to_datetime(fund.fund_end_time).to_rfc3339(),
        unix_timestamp_to_datetime(fund.next_fund_start_time).to_rfc3339(),
    ]
}

fn convert_voteplan(voteplan: &Voteplan) -> Vec<String> {
    vec![
        voteplan.id.to_string(),
        voteplan.chain_voteplan_id.to_string(),
        unix_timestamp_to_datetime(voteplan.chain_vote_start_time).to_rfc3339(),
        unix_timestamp_to_datetime(voteplan.chain_vote_end_time).to_rfc3339(),
        unix_timestamp_to_datetime(voteplan.chain_committee_end_time).to_rfc3339(),
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
        challenge.fund_id.to_string(),
        challenge.challenge_url.clone(),
    ]
}
