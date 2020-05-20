pub mod v0;

use std::error::Error;
use std::sync::Arc;
use tokio;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    // TODO: create configuration to load the required context information
    let context = v0::context::new_default_context();
    let app = v0::filter(context);
    // TODO: load serving address and port from configuration
    warp::serve(app).run(([127, 0, 0, 1], 3030)).await;
}
