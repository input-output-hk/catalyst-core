use crate::common::{
    clients::{
        graphql::templates::{
            ProposalAlias, ProposalAliases, ProposalComment, ProposalCycle, ProposalDoesNotExist,
            ProposalFragments, ProposalWithoutArgument, Proposals,
        },
        GraphqlClient,
    },
    data::Snapshot,
    startup::quick_start,
};
use askama::Template;
use assert_fs::TempDir;

pub fn get_proposal_by_id_test(
    graphql_client: &GraphqlClient,
    proposal_id: u32,
    snapshot: &Snapshot,
) {
    let proposal = graphql_client.proposal_by_id(proposal_id).unwrap();
    assert_eq!(
        proposal,
        snapshot
            .proposal_by_id(&proposal_id.to_string())
            .unwrap()
            .clone()
    );
}

pub fn run_query_with_comment_test(graphql_client: &GraphqlClient, proposal_id: u32) {
    let query_result = graphql_client
        .run_query(&ProposalComment { id: proposal_id }.render().unwrap())
        .unwrap();
    assert!(query_result["errors"][0]["message"]
        .as_str()
        .unwrap()
        .contains("expected selection"));
}

pub fn additional_field_test(graphql_client: &GraphqlClient, proposal_id: u32) {
    let query_result = graphql_client
        .run_query(&ProposalDoesNotExist { id: proposal_id }.render().unwrap())
        .unwrap();
    assert!(query_result["errors"][0]["message"]
        .as_str()
        .unwrap()
        .contains("Unknown field"));
}

pub fn aliases_test(graphql_client: &GraphqlClient, proposal_id: u32) {
    let query_result = graphql_client
        .run_query(&ProposalAliases { id: proposal_id }.render().unwrap())
        .unwrap();
    assert!(query_result["errors"][0]["message"]
        .as_str()
        .unwrap()
        .contains("This anonymous operation must be the only defined operation"));
}

pub fn alias_test(graphql_client: &GraphqlClient, proposal_id: u32) {
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
}

pub fn cycle_test(graphql_client: &GraphqlClient, proposal_id: u32) {
    let query_result = graphql_client
        .run_query(&ProposalCycle { id: proposal_id }.render().unwrap())
        .unwrap();
    assert!(query_result["errors"][0]["message"]
        .as_str()
        .unwrap()
        .contains("expected selection"));
}

pub fn proposal_without_arguments_test(graphql_client: &GraphqlClient) {
    let query_result = graphql_client
        .run_query(&ProposalWithoutArgument.render().unwrap())
        .unwrap();
    assert!(query_result["errors"][0]["message"].as_str().unwrap().contains("Field \"proposal\" argument \"proposalId\" of type \"QueryRoot\" is required but not provided"));
}

pub fn proposals_test(graphql_client: &GraphqlClient, snapshot: &Snapshot) {
    let query_result = graphql_client
        .run_query(&Proposals.render().unwrap())
        .unwrap();
    assert_eq!(
        query_result["data"]["proposals"].as_array().unwrap().len(),
        snapshot.proposals().len()
    );
}

pub fn proposal_with_fragments_test(graphql_client: &GraphqlClient, proposal_id: u32) {
    let query_result = graphql_client
        .run_query(&ProposalFragments { id: proposal_id }.render().unwrap())
        .unwrap();
    assert!(query_result["errors"][0]["message"]
        .as_str()
        .unwrap()
        .contains("Unknown fragment"));
}

#[test]
pub fn test_graphql_requests() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let proposal_id: u32 = snapshot
        .proposals()
        .first()
        .unwrap()
        .proposal_id
        .parse()
        .unwrap();

    let graphql_client = server.graphql_client_with_token(&snapshot.token_hash());

    get_proposal_by_id_test(&graphql_client, proposal_id, &snapshot);

    run_query_with_comment_test(&graphql_client, proposal_id);

    additional_field_test(&graphql_client, proposal_id);

    aliases_test(&graphql_client, proposal_id);

    alias_test(&graphql_client, proposal_id);

    cycle_test(&graphql_client, proposal_id);

    proposal_without_arguments_test(&graphql_client);

    proposals_test(&graphql_client, &snapshot);

    proposal_with_fragments_test(&graphql_client, proposal_id);

    Ok(())
}
