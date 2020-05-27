#[macro_use]
extern crate diesel;

pub mod db;
pub mod server;
pub mod settings;
pub mod v0;

#[tokio::main]
async fn main() {
    // TODO: create configuration to load the required context information
    let context = v0::context::new_default_context();
    let app = v0::filter(context);
    // TODO: load serving address and port from configuration
    warp::log("Running server at 127.0.0.1:3030");
    server::start_server(app, None).await
}
