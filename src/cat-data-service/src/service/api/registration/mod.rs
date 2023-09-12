use crate::{poem_types::registration::VotingKey, service::common::tags::ApiTags, state::State};
use poem::web::{Data, Path};
use poem_openapi::OpenApi;
use std::sync::Arc;

mod voter_info_get;

pub struct RegistrationApi;

#[OpenApi(prefix_path = "/registration", tag = "ApiTags::Registration")]
impl RegistrationApi {
    #[oai(
        path = "/voter/:voting_key",
        method = "get",
        operation_id = "getVoterInfo"
    )]
    /// Voter's info
    ///
    /// Get voter's registration and voting power by their voting key.
    /// If the `event_id` query parameter is omitted, then the latest voting power is retrieved.
    /// If the `with_delegators` query parameter is ommitted, then `delegator_addresses` field of `VoterInfo` type does not provided.
    ///
    async fn get_voter_info(
        &self,
        pool: Data<&Arc<State>>,
        path: Path<VotingKey>,
    ) -> voter_info_get::AllResponses {
        voter_info_get::endpoint(pool.0, path.0).await
    }
}
