pub mod settings;
pub mod v0;

use crate::settings::ServiceSettings;
use crate::v0::context::SharedContext;

use tokio;
use warp::Filter;

async fn start_server<App>(app: App)
where
    App: Filter<Error = warp::Rejection> + Clone + Send + Sync + 'static,
    App::Extract: warp::Reply,
{
    warp::serve(app).run(([127, 0, 0, 1], 3030)).await;
}

#[tokio::main]
async fn main() {
    // TODO: create configuration to load the required context information
    let context = v0::context::new_default_context();
    let app = v0::filter(context);
    // TODO: load serving address and port from configuration
    start_server(app).await
}
