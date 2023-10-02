use crate::common::data::{
    ArbitraryValidVotingTemplateGenerator, CurrentFund, ValidVotePlanGenerator,
    ValidVotePlanParameters,
};
use crate::common::startup::{db::DbBuilder, server::ServerBootstrapper};
use assert_fs::TempDir;
use chain_impl_mockchain::testing::scenario::template::VotePlanDef;
use chain_impl_mockchain::testing::VoteTestGen;

#[test]
pub fn bootstrap_with_valid_data() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap().into_persistent();
    let vote_plan = ValidVotePlanParameters::from(CurrentFund::from_single(
        VotePlanDef::from_vote_plan(
            "test",
            Some("owner"),
            &VoteTestGen::vote_plan_with_proposals(30),
        ),
        Default::default(),
    ));
    let snapshot = ValidVotePlanGenerator::new(vote_plan)
        .build(&mut ArbitraryValidVotingTemplateGenerator::new());
    let db_path = DbBuilder::new().with_snapshot(&snapshot).build(&temp_dir)?;

    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .start(&temp_dir)?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    assert!(server.is_up(&snapshot.any_token().0));

    let rest_client = server.rest_client_with_token(&snapshot.token_hash());
    assert!(rest_client.proposals().is_ok());
    assert!(rest_client.challenges().is_ok());
    assert!(rest_client.funds().is_ok());
    Ok(())
}
