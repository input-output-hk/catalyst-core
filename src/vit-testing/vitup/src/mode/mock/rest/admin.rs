use crate::mode::mock::ContextLock;
use tracing::info;
use vit_servicing_station_lib::db::models::funds::Fund;
use warp::{Filter, Rejection, Reply};

pub fn admin_filter(
    context: ContextLock,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let base = warp::path!("admin" / ..);

    let fund_filter = warp::path!("fund" / ..).and(fund_put_filter(context));

    base.and(fund_filter)
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
