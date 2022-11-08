mod advisor_reviews;
pub mod challenges;
mod funds;
mod genesis;
mod health;
pub mod proposals;
pub mod search;
pub mod service_version;
pub mod snapshot;
mod votes;

use crate::v0::context::SharedContext;

use crate::v0::api_token;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub async fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
    enable_api_tokens: bool,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    // mount health endpoint
    let health_root = warp::path!("health" / ..);
    let health_filter = health::filter(health_root.boxed(), context.clone()).await;

    // mount chain-data endpoint
    let chain_data_root = warp::path!("proposals" / ..);
    let chain_data_filter =
        proposals::proposals_filter(chain_data_root.boxed(), context.clone()).await;

    // mount single proposal endpoint
    let proposal_filter =
        proposals::proposal_filter((warp::path!("proposal" / ..)).boxed(), context.clone()).await;

    // mount funds endpoint
    let funds_root = warp::path!("fund" / ..);
    let funds_filter = funds::filter(funds_root.boxed(), context.clone()).await;

    // mount challenges endpoint
    let challenges_root = warp::path!("challenges" / ..);
    let challenges_filter = challenges::filter(challenges_root.boxed(), context.clone()).await;

    // mount genesis endpoint
    let genesis_root = warp::path!("block0" / ..);
    let genesis_filter = genesis::filter(genesis_root.boxed(), context.clone());

    let reviews_root = warp::path!("reviews" / ..);
    let reviews_filter = advisor_reviews::filter(reviews_root.boxed(), context.clone()).await;

    let votes_root = warp::path!("votes" / ..);
    let votes_filter = votes::filter(votes_root.boxed(), context.clone()).await;

    let search_root = warp::path!("search" / ..);
    let search_filter = search::search_filter(search_root.boxed(), context.clone()).await;

    let search_count_root = warp::path!("search_count" / ..);
    let search_count_filter =
        search::search_count_filter(search_count_root.boxed(), context.clone()).await;

    let snapshot_root = warp::path!("snapshot" / ..);
    let snapshot_rx_filter = snapshot::filter(snapshot_root.boxed(), context.clone());

    let admin_filter = {
        let base = warp::path!("admin" / ..);

        let snapshot_tx_filter =
            warp::path!("snapshot" / ..).and(snapshot::update_filter(context.clone()).boxed());

        let fund_filter = warp::path!("fund" / ..).and(funds::admin_filter(context.clone()));

        base.and(snapshot_tx_filter.or(fund_filter))
    };

    let api_token_filter = if enable_api_tokens {
        api_token::api_token_filter(context).await.boxed()
    } else {
        warp::any().boxed()
    };

    root.and(
        api_token_filter.and(
            health_filter
                .or(genesis_filter)
                .or(chain_data_filter)
                .or(proposal_filter)
                .or(funds_filter)
                .or(challenges_filter)
                .or(reviews_filter)
                .or(votes_filter)
                .or(search_filter)
                .or(search_count_filter)
                .or(snapshot_rx_filter)
                .or(admin_filter),
        ),
    )
    .boxed()
}
