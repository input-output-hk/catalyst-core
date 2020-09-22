use jortestkit::csv::CsvFileBuilder;
use std::path::Path;
use thiserror::Error;
use vit_servicing_station_lib::db::models::{
    funds::Fund, proposals::Proposal, voteplans::Voteplan,
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
            "fund_id",
        ];
        let content: Vec<Vec<String>> = voteplans.iter().map(|x| convert_voteplan(x)).collect();
        self.build_file(headers, content, path)
    }

    pub fn proposals<P: AsRef<Path>>(
        &self,
        proposals: Vec<Proposal>,
        path: P,
    ) -> Result<(), Error> {
        let headers = vec![
            "internal_id",
            "category_name",
            "proposal_id",
            "proposal_title",
            "proposal_summary",
            "proposal_problem",
            "proposal_solution",
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
        ];

        let content: Vec<Vec<String>> = proposals.iter().map(|x| convert_proposal(x)).collect();
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

fn convert_proposal(proposal: &Proposal) -> Vec<String> {
    vec![
        proposal.internal_id.to_string(),
        proposal.proposal_category.category_name.to_string(),
        proposal.proposal_id.to_string(),
        proposal.proposal_title.to_string(),
        proposal.proposal_summary.to_string(),
        proposal.proposal_problem.to_string(),
        proposal.proposal_solution.to_string(),
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
        proposal.chain_vote_start_time.to_string(),
        proposal.chain_vote_end_time.to_string(),
        proposal.chain_committee_end_time.to_string(),
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
        fund.fund_start_time.to_string(),
        fund.fund_end_time.to_string(),
        fund.next_fund_start_time.to_string(),
    ]
}

fn convert_voteplan(voteplan: &Voteplan) -> Vec<String> {
    vec![
        voteplan.id.to_string(),
        voteplan.chain_voteplan_id.to_string(),
        voteplan.chain_vote_start_time.to_string(),
        voteplan.chain_vote_end_time.to_string(),
        voteplan.chain_committee_end_time.to_string(),
        voteplan.chain_voteplan_payload.to_string(),
        voteplan.fund_id.to_string(),
    ]
}
