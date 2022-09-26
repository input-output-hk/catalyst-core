use super::VoterInfo;
use crate::v0::context::SharedContext;
use crate::v0::result::HandlerResult;
use jormungandr_lib::crypto::account::Identifier;
use jormungandr_lib::interfaces::Value;
use serde::{Deserialize, Serialize};
use serde_json::json;
use snapshot_lib::{Fraction, RawSnapshot, SnapshotInfo};
use time::OffsetDateTime;
use warp::http::StatusCode;
use warp::{Rejection, Reply};

#[tracing::instrument(skip(context))]
pub async fn get_voters_info(
    tag: String,
    voting_key: String,
    context: SharedContext,
) -> Result<impl Reply, Rejection> {
    let key = if let Ok(key) = Identifier::from_hex(&voting_key) {
        key
    } else {
        return Ok(warp::reply::with_status(
            "Invalid voting key",
            StatusCode::UNPROCESSABLE_ENTITY,
        )
        .into_response());
    };

    match super::get_voters_info(&tag, &key, context).await {
        Ok(snapshot) => {
            let voter_info: Vec<_> = snapshot.voter_info.into_iter().map(|VoterInfo{voting_group, voting_power,delegations_power, delegations_count}| {
            json!({"voting_power": voting_power, "voting_group": voting_group, "delegations_power": delegations_power, "delegations_count": delegations_count})
        }).collect();
            if let Ok(last_update) = OffsetDateTime::from_unix_timestamp(snapshot.last_updated) {
                let results =
                    json!({"voter_info": voter_info, "last_updated": last_update.unix_timestamp()});
                Ok(warp::reply::json(&results).into_response())
            } else {
                Ok(
                    warp::reply::with_status("Invalid time", StatusCode::UNPROCESSABLE_ENTITY)
                        .into_response(),
                )
            }
        }
        Err(err) => Ok(err.into_response()),
    }
}

#[tracing::instrument(skip(context))]
pub async fn get_tags(context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(super::get_tags(context).await))
}

/// Snapshot information update with timestamp.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct SnapshotInfoInput {
    pub snapshot: Vec<SnapshotInfo>,
    pub update_timestamp: u64,
}

/// Raw Snapshot information update with timestamp.
#[derive(Debug, Serialize, Deserialize)]
pub struct RawSnapshotInput {
    pub snapshot: RawSnapshot,
    pub update_timestamp: u64,
    pub min_stake_threshold: Value,
    pub voting_power_cap: Fraction,
    pub direct_voters_group: Option<String>,
    pub representatives_group: Option<String>,
}

#[tracing::instrument(skip(context))]
pub async fn put_raw_snapshot(
    tag: String,
    input: RawSnapshotInput,
    context: SharedContext,
) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(
        super::update_from_raw_snapshot(
            &tag,
            input.snapshot,
            input.update_timestamp,
            input.min_stake_threshold,
            input.voting_power_cap,
            input.direct_voters_group,
            input.representatives_group,
            context,
        )
        .await,
    ))
}

#[tracing::instrument(skip(context))]
pub async fn put_snapshot_info(
    tag: String,
    input: SnapshotInfoInput,
    context: SharedContext,
) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(
        super::update_from_shanpshot_info(&tag, input.snapshot, input.update_timestamp, context)
            .await,
    ))
}
