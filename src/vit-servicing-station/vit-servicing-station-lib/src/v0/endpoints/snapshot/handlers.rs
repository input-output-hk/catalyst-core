use super::{SharedContext, UpdateHandle, VoterInfo};
use jormungandr_lib::crypto::account::Identifier;
use jormungandr_lib::interfaces::Value;
use serde::{Deserialize, Serialize};
use serde_json::json;
use snapshot_lib::{Fraction, RawSnapshot, SnapshotInfo};
use std::sync::Arc;
use time::OffsetDateTime;
use tokio::sync::Mutex;
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

    match tokio::task::spawn_blocking(move || context.get_voters_info(&tag, &key))
        .await
        .unwrap()
    {
        Ok(Some(snapshot)) => {
            let voter_info: Vec<_> = snapshot.voter_info.into_iter().map(|VoterInfo{voting_group, voting_power,delegations_power, delegations_count}| {
            json!({"voting_power": voting_power, "voting_group": voting_group, "delegations_power": delegations_power, "delegations_count": delegations_count})
        }).collect();
            if let Ok(last_update) =
                OffsetDateTime::from_unix_timestamp(snapshot.last_updated.try_into().unwrap())
            {
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
        Ok(None) => Err(warp::reject::not_found()),
        Err(_) => Ok(
            warp::reply::with_status("Database error", StatusCode::INTERNAL_SERVER_ERROR)
                .into_response(),
        ),
    }
}

#[tracing::instrument(skip(context))]
pub async fn get_tags(context: SharedContext) -> Result<impl Reply, Rejection> {
    match context.get_tags().map(|tags| warp::reply::json(&tags)) {
        Ok(tags) => Ok(tags.into_response()),
        Err(_) => Ok(warp::reply::with_status(
            "Failed to get tags from database",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response()),
    }
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
    context: Arc<Mutex<UpdateHandle>>,
) -> Result<impl Reply, Rejection> {
    let mut handle = context.lock().await;

    match handle
        .update_from_raw_snapshot(
            &tag,
            input.snapshot,
            input.update_timestamp,
            input.min_stake_threshold,
            input.voting_power_cap,
            input.direct_voters_group,
            input.representatives_group,
        )
        .await
    {
        Err(super::Error::InternalError) => Ok(warp::reply::with_status(
            "Consistency error",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response()),
        Err(e) => Ok(
            warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
                .into_response(),
        ),
        Ok(_) => Ok(warp::reply().into_response()),
    }
}

#[tracing::instrument(skip(context))]
pub async fn put_snapshot_info(
    tag: String,
    input: SnapshotInfoInput,
    context: Arc<Mutex<UpdateHandle>>,
) -> Result<impl Reply, Rejection> {
    let mut handle = context.lock().await;

    match handle
        .update_from_shanpshot_info(&tag, input.snapshot, input.update_timestamp)
        .await
    {
        Err(super::Error::InternalError) => Ok(warp::reply::with_status(
            "Consistency error",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response()),
        Err(e) => Ok(
            warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
                .into_response(),
        ),
        Ok(_) => Ok(warp::reply().into_response()),
    }
}
