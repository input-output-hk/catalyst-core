use vit_servicing_station_lib::db::models::{
    api_tokens::ApiTokenData, challenges::Challenge, funds::Fund, proposals::FullProposalInfo,
    voteplans::Voteplan,
};

mod csv_converter;
mod generator;
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
