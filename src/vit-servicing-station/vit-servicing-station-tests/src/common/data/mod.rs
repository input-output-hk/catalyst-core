use rand_core::OsRng;
use rand_core::RngCore;
use vit_servicing_station_lib::db::models::{
    api_tokens::APITokenData, funds::Fund, proposals::Proposal, voteplans::Voteplan,
};

fn bytes() -> [u8; 32] {
    let mut random_bytes: [u8; 32] = [0; 32];
    let mut rng = OsRng;
    rng.fill_bytes(&mut random_bytes);
    random_bytes
}

pub fn invalid_token_hash() -> String {
    base64::encode_config(bytes().to_vec(), base64::URL_SAFE_NO_PAD)
}

mod generator;
pub use generator::{Generator, Snapshot};

pub fn token() -> (APITokenData, String) {
    Generator::new().token()
}

pub fn proposals() -> Vec<Proposal> {
    let mut gen = Generator::new();
    let funds = gen.funds();
    gen.proposals(&funds)
}

pub fn funds() -> Vec<Fund> {
    Generator::new().funds()
}

pub fn voteplans() -> Vec<Voteplan> {
    let mut gen = Generator::new();
    let funds = gen.funds();
    gen.voteplans(&funds)
}
