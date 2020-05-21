pub mod server;
pub mod settings;
pub mod v0;

#[tokio::main]
async fn main() {
    // TODO: create configuration to load the required context information
    let context = v0::context::new_default_context();
    let app = v0::filter(context);
    // TODO: load serving address and port from configuration
    server::start_server(app, None).await
}
