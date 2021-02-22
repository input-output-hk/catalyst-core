use crate::common::{
    clients::graphql::templates::{
        ProposalAlias, ProposalAliases, ProposalComment, ProposalCycle, ProposalDoesNotExist,
        ProposalFragments, ProposalWithoutArgument, Proposals,
    },
    startup::quick_start,
};
use askama::Template;
use assert_fs::TempDir;

#[test]
#[ignore]
pub fn get_proposal_by_id_test() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let proposal_id: u32 = snapshot
        .proposals()
        .first()
        .unwrap()
        .proposal
        .proposal_id
        .parse()
        .unwrap();

    let graphql_client = server.graphql_client_with_token(&snapshot.token_hash());

    let proposal = graphql_client.proposal_by_id(proposal_id).unwrap();
    assert_eq!(
        proposal,
        snapshot
            .proposal_by_id(&proposal_id.to_string())
            .unwrap()
            .clone()
            .proposal
    );
    Ok(())
}

#[test]
pub fn run_query_with_comment_test() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let proposal_id: u32 = snapshot
        .proposals()
        .first()
        .unwrap()
        .proposal
        .proposal_id
        .parse()
        .unwrap();

    let graphql_client = server.graphql_client_with_token(&snapshot.token_hash());
    let query_result = graphql_client
        .run_query(&ProposalComment { id: proposal_id }.render().unwrap())
        .unwrap();
    assert!(query_result["errors"][0]["message"]
        .as_str()
        .unwrap()
        .contains("expected selection"));
    Ok(())
}

#[test]
pub fn additional_field_test() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let proposal_id: u32 = snapshot
        .proposals()
        .first()
        .unwrap()
        .proposal
        .proposal_id
        .parse()
        .unwrap();

    let graphql_client = server.graphql_client_with_token(&snapshot.token_hash());
    let query_result = graphql_client
        .run_query(&ProposalDoesNotExist { id: proposal_id }.render().unwrap())
        .unwrap();
    assert!(query_result["errors"][0]["message"]
        .as_str()
        .unwrap()
        .contains("Unknown field"));
    Ok(())
}

#[test]
#[ignore]
pub fn aliases_test() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let proposal_id: u32 = snapshot
        .proposals()
        .first()
        .unwrap()
        .proposal
        .proposal_id
        .parse()
        .unwrap();

    let graphql_client = server.graphql_client_with_token(&snapshot.token_hash());
    let query_result = graphql_client
        .run_query(&ProposalAliases { id: proposal_id }.render().unwrap())
        .unwrap();
    assert!(query_result["errors"][0]["message"]
        .as_str()
        .unwrap()
        .contains("This anonymous operation must be the only defined operation"));
    Ok(())
}

#[test]
pub fn alias_test() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let proposal_id: u32 = snapshot
        .proposals()
        .first()
        .unwrap()
        .proposal
        .proposal_id
        .parse()
        .unwrap();

    let graphql_client = server.graphql_client_with_token(&snapshot.token_hash());
    let query_result = graphql_client
        .run_query(&ProposalAlias { id: proposal_id }.render().unwrap())
        .unwrap();
    assert_eq!(
        query_result["data"]["query_1"]["proposalId"]
            .as_str()
            .unwrap()
            .to_owned(),
        proposal_id.to_string()
    );
    Ok(())
}

#[test]
pub fn cycle_test() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let proposal_id: u32 = snapshot
        .proposals()
        .first()
        .unwrap()
        .proposal
        .proposal_id
        .parse()
        .unwrap();

    let graphql_client = server.graphql_client_with_token(&snapshot.token_hash());
    let query_result = graphql_client
        .run_query(&ProposalCycle { id: proposal_id }.render().unwrap())
        .unwrap();
    assert!(query_result["errors"][0]["message"]
        .as_str()
        .unwrap()
        .contains("expected selection"));
    Ok(())
}

#[test]
pub fn proposal_without_arguments_test() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let graphql_client = server.graphql_client_with_token(&snapshot.token_hash());
    let query_result = graphql_client
        .run_query(&ProposalWithoutArgument.render().unwrap())
        .unwrap();
    assert!(query_result["errors"][0]["message"].as_str().unwrap().contains("Field \"proposal\" argument \"proposalId\" of type \"QueryRoot\" is required but not provided"));
    Ok(())
}

#[test]
#[ignore]
pub fn proposals_test() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let graphql_client = server.graphql_client_with_token(&snapshot.token_hash());
    let query_result = graphql_client
        .run_query(&Proposals.render().unwrap())
        .unwrap();
    assert_eq!(
        query_result["data"]["proposals"].as_array().unwrap().len(),
        snapshot.proposals().len()
    );
    Ok(())
}

#[test]
pub fn proposal_with_fragments_test() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let proposal_id: u32 = snapshot
        .proposals()
        .first()
        .unwrap()
        .proposal
        .proposal_id
        .parse()
        .unwrap();

    let graphql_client = server.graphql_client_with_token(&snapshot.token_hash());
    let query_result = graphql_client
        .run_query(&ProposalFragments { id: proposal_id }.render().unwrap())
        .unwrap();
    assert!(query_result["errors"][0]["message"]
        .as_str()
        .unwrap()
        .contains("Unknown fragment"));
    Ok(())
}
