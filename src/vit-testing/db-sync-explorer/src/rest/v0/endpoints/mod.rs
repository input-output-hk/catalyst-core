use crate::rest::v0::context::SharedContext;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

mod api_token;
mod health;
mod meta;
mod mock;
mod sync;
mod tx;

pub async fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let sync_root = warp::path!("sync" / ..);
    let sync_filter = sync::filter(sync_root.boxed(), context.clone()).await;

    let health_root = warp::path!("health" / ..);
    let health_filter = health::filter(health_root.boxed(), context.clone()).await;

    let tx_root = warp::path!("tx" / ..);
    let tx_filter = tx::filter(tx_root.boxed(), context.clone()).await;

    let meta_root = warp::path!("meta" / ..);
    let meta_filter = meta::filter(meta_root.boxed(), context.clone()).await;

    let mock_root = warp::path!("mock" / ..);
    let mock_filter = mock::filter(mock_root.boxed(), context.clone()).await;

    let is_token_enabled = context.read().await.token().is_some();

    let api_token_filter = if is_token_enabled {
        api_token::api_token_filter(context).await.boxed()
    } else {
        warp::any().boxed()
    };

    root.and(
        api_token_filter.and(
            sync_filter
                .or(health_filter)
                .or(tx_filter)
                .or(meta_filter)
                .or(mock_filter),
        ),
    )
    .boxed()
}
