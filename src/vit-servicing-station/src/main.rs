use warp::Filter;
use tokio;

#[tokio::main]
async fn main() {
    let hello = warp::path!("hello")
        .map(|| "Hello IOHK!");

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
