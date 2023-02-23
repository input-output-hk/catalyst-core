use crate::v0::context::SharedContext;
use crate::v0::result::HandlerResult;
use jormungandr_lib::interfaces::Value;
use serde::{Deserialize, Serialize};
use snapshot_lib::{Dreps, Fraction, RawSnapshot, SnapshotInfo};
use warp::{Rejection, Reply};

#[tracing::instrument(skip(context))]
pub async fn get_voters_info(
    tag: String,
    voting_key: String,
    context: SharedContext,
) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(
        super::get_voters_info(tag, voting_key, context).await,
    ))
}

#[tracing::instrument(skip(context))]
pub async fn get_delegator_info(
    tag: String,
    stake_public_key: String,
    context: SharedContext,
) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(
        super::get_delegator_info(tag, stake_public_key, context).await,
    ))
}

#[tracing::instrument(skip(context))]
pub async fn get_tags(context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(super::get_tags(context).await))
}

/// Snapshot information update with timestamp.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct SnapshotInfoInput {
    pub snapshot: Vec<SnapshotInfo>,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub update_timestamp: i64,
}

/// Raw Snapshot information update with timestamp.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawSnapshotInput {
    pub snapshot: RawSnapshot,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub update_timestamp: i64,
    pub min_stake_threshold: Value,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_fraction_from_string")]
    #[serde(serialize_with = "crate::utils::serde::serialize_fraction_to_string")]
    pub voting_power_cap: Fraction,
    pub direct_voters_group: Option<String>,
    pub representatives_group: Option<String>,
    pub dreps: Option<Dreps>,
}

#[tracing::instrument(skip(context))]
pub async fn put_raw_snapshot(
    tag: String,
    input: RawSnapshotInput,
    context: SharedContext,
) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(
        super::update_from_raw_snapshot(
            tag,
            input.snapshot,
            input.update_timestamp,
            input.min_stake_threshold,
            input.voting_power_cap,
            input.direct_voters_group,
            input.representatives_group,
            input.dreps,
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
        super::update_from_snapshot_info(tag, input.snapshot, input.update_timestamp, context)
            .await,
    ))
}
