use vit_servicing_station_lib::db::models::{
    api_tokens::APITokenData, funds::Fund, proposals::Proposal, voteplans::Voteplan,
};

mod csv_converter;
mod generator;
pub use csv_converter::CsvConverter;
pub use generator::{
    ArbitraryGenerator, ArbitraryValidVotingTemplateGenerator,
    ExternalValidVotingTemplateGenerator, Snapshot, ValidVotePlanGenerator,
    ValidVotingTemplateGenerator,ValidVotePlanParameters
};

pub fn token() -> (String, APITokenData) {
    ArbitraryGenerator::new().token()
}

pub fn token_hash() -> String {
    token().0
}

pub fn proposals() -> Vec<Proposal> {
    let mut gen = ArbitraryGenerator::new();
    let funds = gen.funds();
    gen.proposals(&funds)
}

pub fn funds() -> Vec<Fund> {
    ArbitraryGenerator::new().funds()
}

pub fn voteplans() -> Vec<Voteplan> {
    let mut gen = ArbitraryGenerator::new();
    let funds = gen.funds();
    gen.voteplans(&funds)
}
