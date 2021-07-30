use crate::common::{
    data,
    startup::{db::DbBuilder, server::ServerBootstrapper},
};
use assert_fs::TempDir;
use reqwest::StatusCode;
use vit_servicing_station_lib::db::models::community_advisors_reviews::AdvisorReview;

#[test]
pub fn get_advisor_reviews() -> Result<(), Box<dyn std::error::Error>> {
    use pretty_assertions::assert_eq;
    let temp_dir = TempDir::new().unwrap().into_persistent();
    let proposal_id = 1234;
    let expected_review = AdvisorReview {
        id: 0,
        proposal_id,
        rating_given: 0,
        assessor: "za_foo_bar".to_string(),
        note: "foo bar".to_string(),
    };
    let (hash, token) = data::token();

    let db_path = DbBuilder::new()
        .with_token(token)
        .with_advisor_reviews(vec![expected_review.clone()])
        .build(&temp_dir)?;

    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .start(&temp_dir)?;

    let rest_client = server.rest_client_with_token(&hash);

    let actual_review = rest_client.advisor_reviews(&expected_review.proposal_id.to_string())?;
    assert_eq!(expected_review, actual_review[0]);

    // non existing
    let empty_reviews = rest_client.advisor_reviews("0")?;
    assert!(empty_reviews.is_empty());
    // malformed index
    assert_eq!(
        rest_client.advisor_reviews_raw("a")?.status(),
        StatusCode::NOT_FOUND
    );
    // overflow index
    assert_eq!(
        rest_client.fund_raw("3147483647999")?.status(),
        StatusCode::NOT_FOUND
    );

    Ok(())
}
