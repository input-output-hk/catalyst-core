pub mod v0;

use tokio;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let context = v0::context::new_default_context();
    let app = v0::filter(context);
    warp::serve(app).run(([127, 0, 0, 1], 3030)).await;
}
