use crate::mode::mock::ContextLock;
use snapshot_lib::voting_group::{
    RepsVotersAssigner, DEFAULT_DIRECT_VOTER_GROUP, DEFAULT_REPRESENTATIVE_GROUP,
};
use snapshot_lib::{Snapshot, SnapshotInfo};
use tracing::info;
use vit_servicing_station_lib::db::models::funds::Fund;
use vit_servicing_station_lib::v0::endpoints::snapshot::{
    convert_snapshot_to_contrib, RawSnapshotInput, SnapshotInfoInput,
};
use vit_servicing_station_lib::v0::errors::HandleError;
use vit_servicing_station_lib::v0::result::HandlerResult;
use warp::{Filter, Rejection, Reply};

pub fn admin_filter(
    context: ContextLock,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let base = warp::path!("admin" / ..);

    let snapshot_tx_filter =
        warp::path!("snapshot" / ..).and(snapshot_update_filter(context.clone()).boxed());

    let fund_filter = warp::path!("fund" / ..).and(fund_put_filter(context));

    base.and(snapshot_tx_filter.or(fund_filter))
}

pub fn snapshot_update_filter(
    context: ContextLock,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    let snapshot_info = warp::path!("snapshot_info" / String)
        .and(warp::put())
        .and(warp::body::json())
        .and(with_context.clone())
        .and_then(put_snapshot_info);

    let raw_snapshot = warp::path!("raw_snapshot" / String)
        .and(warp::put())
        .and(warp::body::json())
        .and(with_context)
        .and_then(put_raw_snapshot);

    snapshot_info.or(raw_snapshot)
}

#[tracing::instrument(skip(context, input), name = "mock admin command received")]
pub async fn put_raw_snapshot(
    tag: String,
    input: RawSnapshotInput,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    info!("put raw snapshot");

    let direct_voter = input
        .direct_voters_group
        .unwrap_or_else(|| DEFAULT_DIRECT_VOTER_GROUP.to_owned());
    let representative = input
        .representatives_group
        .unwrap_or_else(|| DEFAULT_REPRESENTATIVE_GROUP.to_owned());
    let assigner = RepsVotersAssigner::new(
        direct_voter,
        representative,
        input.dreps.unwrap_or_default(),
    );
    let snapshot = Snapshot::from_raw_snapshot(
        input.snapshot,
        input.min_stake_threshold,
        input.voting_power_cap,
        &assigner,
    )
    .map_err(|e| HandleError::InternalError(e.to_string()))?
    .to_full_snapshot_info();

    update_from_snapshot_info(tag, snapshot, input.update_timestamp, context).await?;

    Ok(warp::reply())
}

#[tracing::instrument(skip(context, input), name = "mock admin command received")]
pub async fn put_snapshot_info(
    tag: String,
    input: SnapshotInfoInput,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    info!("put snapshot");

    Ok(HandlerResult(
        update_from_snapshot_info(tag, input.snapshot, input.update_timestamp, context).await,
    ))
}

#[tracing::instrument(skip(snapshot, context), name = "mock admin command received")]
pub async fn update_from_snapshot_info(
    tag: String,
    snapshot: impl IntoIterator<Item = SnapshotInfo>,
    update_timestamp: i64,
    context: ContextLock,
) -> Result<(), HandleError> {
    info!("update from snapshot info");

    let mut context = context.write().unwrap();
    let state = context.state_mut();
    let voters_mut = state.voters_mut();
    voters_mut.put_snapshot_tag(tag.clone(), update_timestamp);

    let (voters, contributions) = convert_snapshot_to_contrib(tag, snapshot);
    voters_mut.insert_voters(voters);
    voters_mut.insert_contributions(contributions);
    Ok(())
}

pub fn fund_put_filter(
    context: ContextLock,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    warp::path::end()
        .and(warp::put())
        .and(warp::body::json())
        .and(with_context)
        .and_then(put_fund)
}

#[tracing::instrument(skip(context), fields(fund_id = fund.id), name="mock admin command received")]
pub async fn put_fund(fund: Fund, context: ContextLock) -> Result<impl Reply, Rejection> {
    info!("put new fund");

    context
        .write()
        .unwrap()
        .state_mut()
        .vit_mut()
        .funds_mut()
        .push(fund);
    Ok(warp::reply())
}
