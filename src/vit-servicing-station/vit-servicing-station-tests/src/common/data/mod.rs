use chrono::offset::Utc;
use vit_servicing_station_lib::{
    db::models::{
        api_tokens::APITokenData,
        proposals::{Category, Proposal, Proposer},
        vote_options::VoteOptions,
    },
    v0::api_token::APIToken,
};

pub fn token() -> (APITokenData, String) {
    let data = b"ffffffffffffffffffffffffffffffff".to_vec();
    let token_data = APITokenData {
        token: APIToken::new(data.clone()),
        creation_time: Utc::now().timestamp(),
        expire_time: Utc::now().timestamp(),
    };
    (
        token_data,
        base64::encode_config(data, base64::URL_SAFE_NO_PAD),
    )
}

pub fn proposals() -> Vec<Proposal> {
    vec![Proposal {
        internal_id: 1,
        proposal_id: "1".to_string(),
        proposal_category: Category {
            category_id: "".to_string(),
            category_name: "foo_category_name".to_string(),
            category_description: "".to_string(),
        },
        proposal_title: "the proposal".to_string(),
        proposal_summary: "the proposal summary".to_string(),
        proposal_problem: "the proposal problem".to_string(),
        proposal_solution: "the proposal solution".to_string(),
        proposal_public_key: "pubkey".to_string(),
        proposal_funds: 10000,
        proposal_url: "http://foo.bar".to_string(),
        proposal_files_url: "http://foo.bar/files".to_string(),
        proposer: Proposer {
            proposer_name: "tester".to_string(),
            proposer_email: "tester@tester.tester".to_string(),
            proposer_url: "http://tester.tester".to_string(),
        },
        chain_proposal_id: b"foobar".to_vec(),
        chain_proposal_index: 0,
        chain_vote_options: VoteOptions::parse_coma_separated_value("b,a,r"),
        chain_voteplan_id: "voteplain_id".to_string(),
        chain_vote_start_time: Utc::now().timestamp(),
        chain_vote_end_time: Utc::now().timestamp(),
        chain_committee_end_time: Utc::now().timestamp(),
        chain_voteplan_payload: "none".to_string(),
        fund_id: 1,
    }]
}
