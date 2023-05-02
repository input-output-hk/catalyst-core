mod handlers;
mod routes;

use crate::{
    db::queries::snapshot::{
        query_contributions_by_stake_public_key_and_snapshot_tag,
        query_contributions_by_voting_key_and_voter_group_and_snapshot_tag, query_snapshot_by_tag,
        query_total_voting_power_by_voting_group_and_snapshot_tag,
        query_voters_by_voting_key_and_snapshot_tag,
    },
    v0::{context::SharedContext, errors::HandleError},
};
pub use handlers::{RawSnapshotInput, SnapshotInfoInput};
use itertools::Itertools;
use jormungandr_lib::interfaces::Value;
pub use routes::filter;
use serde::{Deserialize, Serialize};

pub type Tag = String;
pub type Group = String;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VoterInfo {
    pub voting_group: Group,
    pub voting_power: Value,
    pub delegations_power: u64,
    pub delegations_count: u64,
    pub voting_power_saturation: f64,
}

/// Voter information in the current snapshot
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VotersInfo {
    /// A listing of voter information in the current snapshot
    pub voter_info: Vec<VoterInfo>,
    /// Timestamp for the latest update in voter info in the current snapshot
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub last_updated: i64,
}

#[tracing::instrument(skip(context))]
pub async fn get_voters_info(
    tag: String,
    voting_key: String,
    context: SharedContext,
) -> Result<VotersInfo, HandleError> {
    let pool = &context.read().await.db_connection_pool;

    let snapshot = query_snapshot_by_tag(tag.clone(), pool).await?;
    let mut voter_info = Vec::new();
    let voters =
        query_voters_by_voting_key_and_snapshot_tag(voting_key.clone(), tag.clone(), pool).await?;

    for voter in voters {
        let contributors = query_contributions_by_voting_key_and_voter_group_and_snapshot_tag(
            voting_key.clone(),
            voter.voting_group.clone(),
            tag.clone(),
            pool,
        )
        .await?;

        let total_voting_power_per_group =
            query_total_voting_power_by_voting_group_and_snapshot_tag(
                voter.voting_group.clone(),
                tag.clone(),
                pool,
            )
            .await? as f64;

        voter_info.push(VoterInfo {
            voting_power: Value::from(voter.voting_power as u64),
            delegations_count: contributors.len() as u64,
            delegations_power: contributors
                .iter()
                .map(|contributor| contributor.value as u64)
                .sum(),
            voting_group: voter.voting_group,
            voting_power_saturation: if total_voting_power_per_group != 0_f64 {
                voter.voting_power as f64 / total_voting_power_per_group
            } else {
                0_f64
            },
        })
    }

    Ok(VotersInfo {
        voter_info,
        last_updated: snapshot.last_updated,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegatorInfo {
    pub dreps: Vec<String>,
    pub voting_groups: Vec<String>,
    /// Timestamp for the latest update in voter info in the current snapshot
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub last_updated: i64,
}

#[tracing::instrument(skip(context))]
pub async fn get_delegator_info(
    tag: String,
    stake_public_key: String,
    context: SharedContext,
) -> Result<DelegatorInfo, HandleError> {
    let pool = &context.read().await.db_connection_pool;

    let snapshot = query_snapshot_by_tag(tag.clone(), pool).await?;

    let contributions =
        query_contributions_by_stake_public_key_and_snapshot_tag(stake_public_key, tag, pool)
            .await?;

    Ok(DelegatorInfo {
        dreps: contributions
            .iter()
            .map(|contribution| contribution.voting_key.clone())
            .unique()
            .collect(),
        voting_groups: contributions
            .iter()
            .map(|contribution| contribution.voting_group.clone())
            .unique()
            .collect(),
        last_updated: snapshot.last_updated,
    })
}
