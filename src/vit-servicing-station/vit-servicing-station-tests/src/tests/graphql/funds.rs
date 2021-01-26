use crate::common::{
    clients::graphql::templates::{
        FundByIdWrongArgType, FundWithoutArgument, FundsFieldDoesNotExist, FundsRequiredFields,
    },
    startup::quick_start,
};
use askama::Template;
use assert_fs::TempDir;

#[test]
pub fn get_fund_by_id_test() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let fund_id: i32 = snapshot.funds().first().unwrap().id;

    let graphql_client = server.graphql_client_with_token(&snapshot.token_hash());

    let fund = graphql_client.fund_by_id(fund_id).unwrap();
    assert_eq!(fund, snapshot.fund_by_id(fund_id).unwrap().clone());
    Ok(())
}

#[test]
pub fn funds_test() -> Result<(), Box<dyn std::error::Error>> {
    use pretty_assertions::assert_eq;
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();
    let graphql_client = server.graphql_client_with_token(&snapshot.token_hash());
    let mut funds = snapshot.funds();
    funds.sort_by_key(|k| k.id);
    assert_eq!(graphql_client.funds().unwrap(), funds);
    Ok(())
}

#[test]
pub fn fund_without_argument_test() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let graphql_client = server.graphql_client_with_token(&snapshot.token_hash());
    let query_result = graphql_client
        .run_query(&FundWithoutArgument.render().unwrap())
        .unwrap();
    assert!(query_result["errors"][0]["message"]
        .as_str()
        .unwrap()
        .contains("expected name"));
    Ok(())
}

#[test]
pub fn fund_wrong_argument_type_test() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let fund_id: i32 = snapshot.funds().first().unwrap().id;
    let graphql_client = server.graphql_client_with_token(&snapshot.token_hash());
    let query_result = graphql_client
        .run_query(
            &FundByIdWrongArgType {
                id: fund_id.to_string(),
            }
            .render()
            .unwrap(),
        )
        .unwrap();
    assert!(query_result["errors"][0]["message"]
        .as_str()
        .unwrap()
        .contains("Invalid value for argument"));
    Ok(())
}

#[test]
pub fn funds_required_fields_test() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let graphql_client = server.graphql_client_with_token(&snapshot.token_hash());
    let query_result = graphql_client
        .run_query(&FundsRequiredFields.render().unwrap())
        .unwrap();

    assert_eq!(
        query_result["data"]["funds"].as_array().unwrap().len(),
        snapshot.funds().len()
    );
    Ok(())
}

#[test]
pub fn funds_field_does_not_exist_test() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let graphql_client = server.graphql_client_with_token(&snapshot.token_hash());
    let query_result = graphql_client
        .run_query(&FundsFieldDoesNotExist.render().unwrap())
        .unwrap();
    assert!(query_result["errors"][0]["message"]
        .as_str()
        .unwrap()
        .contains("Unknown field"));
    Ok(())
}
