use vit_servicing_station_lib::db::models::{
    api_tokens::ApiTokenData, challenges::Challenge, funds::Fund, proposals::FullProposalInfo,
    voteplans::Voteplan,
};

mod csv_converter;
mod generator;
use chain_impl_mockchain::testing::scenario::template::{ProposalDefBuilder, VotePlanDefBuilder};
use chrono::NaiveDateTime;
pub use csv_converter::CsvConverter;
pub use generator::{
    parse_challenges, parse_funds, parse_proposals, parse_reviews, ArbitraryGenerator,
    ArbitrarySnapshotGenerator, ArbitraryValidVotingTemplateGenerator, ChallengeTemplate,
    ExternalValidVotingTemplateGenerator, FundTemplate, ProposalTemplate, ReviewTemplate, Snapshot,
    TemplateLoad, ValidVotePlanGenerator, ValidVotePlanParameters, ValidVotingTemplateGenerator,
};

pub fn token() -> (String, ApiTokenData) {
    ArbitrarySnapshotGenerator::default().token()
}

pub fn token_hash() -> String {
    token().0
}

pub fn proposals() -> Vec<FullProposalInfo> {
    let mut gen = ArbitrarySnapshotGenerator::default();
    let funds = gen.funds();
    gen.proposals(&funds)
}

pub fn funds() -> Vec<Fund> {
    ArbitrarySnapshotGenerator::default().funds()
}

pub fn voteplans() -> Vec<Voteplan> {
    let mut gen = ArbitrarySnapshotGenerator::default();
    let funds = gen.funds();
    gen.voteplans(&funds)
}

pub fn challenges() -> Vec<Challenge> {
    let mut gen = ArbitrarySnapshotGenerator::default();
    let funds = gen.funds();
    gen.challenges(&funds)
}

pub fn valid_fund_snapshot() -> Snapshot {
    let mut vote_plan_builder = VotePlanDefBuilder::new("fund_3");
    vote_plan_builder.owner("committe_wallet_name");
    vote_plan_builder.vote_phases(1, 2, 3);

    for _ in 0..10 {
        let mut proposal_builder = ProposalDefBuilder::new(
            chain_impl_mockchain::testing::VoteTestGen::external_proposal_id(),
        );
        proposal_builder.options(3);
        proposal_builder.action_off_chain();
        vote_plan_builder.with_proposal(&mut proposal_builder);
    }

    let vote_plan = vote_plan_builder.build();
    let format = "%Y-%m-%d %H:%M:%S";
    let mut parameters = ValidVotePlanParameters::from_single(vote_plan);
    parameters.set_voting_power_threshold(8_000);
    parameters.set_voting_start(
        NaiveDateTime::parse_from_str("2015-09-05 23:56:04", format)
            .unwrap()
            .timestamp(),
    );
    parameters.set_voting_tally_start(
        NaiveDateTime::parse_from_str("2015-09-05 23:56:04", format)
            .unwrap()
            .timestamp(),
    );
    parameters.set_voting_tally_end(
        NaiveDateTime::parse_from_str("2015-09-05 23:56:04", format)
            .unwrap()
            .timestamp(),
    );
    parameters.set_next_fund_start_time(
        NaiveDateTime::parse_from_str("2015-09-12 23:56:04", format)
            .unwrap()
            .timestamp(),
    );
    parameters.set_registration_snapshot_time(
        NaiveDateTime::parse_from_str("2015-09-03 20:00:00", format)
            .unwrap()
            .timestamp(),
    );

    let mut template = ArbitraryValidVotingTemplateGenerator::new();
    let mut generator = ValidVotePlanGenerator::new(parameters);
    generator.build(&mut template)
}
