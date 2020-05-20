mod chain_data;
mod genesis;
use crate::v0::context::SharedContext;

use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let chain_data_root = warp::path!("chain-data" / ..);
    let genesis_root = warp::path!("genesis" / ..);
    let chain_data_filter = chain_data::filter(chain_data_root.boxed(), context.clone());
    let genesis_filter = genesis::filter(genesis_root.boxed(), context.clone());
    root.and(genesis_filter.or(chain_data_filter)).boxed()
}
