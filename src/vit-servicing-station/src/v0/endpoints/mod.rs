mod funds;
mod genesis;
mod graphql;
mod proposals;

use crate::v0::context::SharedContext;

use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub async fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    // mount chain-data endpoint
    let chain_data_root = warp::path!("proposals" / ..);
    let chain_data_filter = proposals::filter(chain_data_root.boxed(), context.clone()).await;

    // mount genesis endpoint
    let genesis_root = warp::path!("genesis" / ..);
    let genesis_filter = genesis::filter(genesis_root.boxed(), context.clone());

    // mount graphql endpoint
    let graphql_root = warp::path!("graphql" / ..);
    let graphql_filter = graphql::filter(graphql_root.boxed(), context).await;

    root.and(genesis_filter.or(chain_data_filter).or(graphql_filter))
        .boxed()
}
