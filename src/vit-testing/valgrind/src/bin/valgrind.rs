use clap::Parser;
use valgrind::{Protocol, ValigrindStartupCommand};
use warp::http::StatusCode;
use warp::Filter;
use warp_reverse_proxy::reverse_proxy_filter;

#[tokio::main]
async fn main() {
    let server_stub = ValigrindStartupCommand::parse().build().unwrap();

    let api = warp::path!("api" / ..);

    let v0 = {
        let root = warp::path!("v0" / ..);

        let snapshot = warp::path!("snapshot" / ..).and(reverse_proxy_filter(
            "".to_string(),
            server_stub.http_vit_address(),
        ));

        let proposals = warp::path!("proposals" / ..).and(reverse_proxy_filter(
            "".to_string(),
            server_stub.http_vit_address(),
        ));

        let challenges = warp::path!("challenges" / ..).and(reverse_proxy_filter(
            "".to_string(),
            server_stub.http_vit_address(),
        ));

        let fund = warp::path!("fund" / ..).and(reverse_proxy_filter(
            "".to_string(),
            server_stub.http_vit_address(),
        ));

        let reviews = warp::path!("reviews" / ..).and(reverse_proxy_filter(
            "".to_string(),
            server_stub.http_vit_address(),
        ));

        let account = warp::path!("account" / ..).and(reverse_proxy_filter(
            "".to_string(),
            server_stub.http_node_address(),
        ));

        let fragment = warp::path!("fragment" / "logs").and(reverse_proxy_filter(
            "".to_string(),
            server_stub.http_node_address(),
        ));

        let message = warp::path!("message" / ..).and(reverse_proxy_filter(
            "".to_string(),
            server_stub.http_node_address(),
        ));

        let settings = warp::path!("settings" / ..).and(reverse_proxy_filter(
            "".to_string(),
            server_stub.http_node_address(),
        ));

        let node = warp::path!("node" / "stats").and(reverse_proxy_filter(
            "".to_string(),
            server_stub.http_node_address(),
        ));

        let explorer = warp::path!("explorer" / "graphql").and(reverse_proxy_filter(
            "".to_string(),
            server_stub.http_node_address(),
        ));

        let vote = warp::path!("vote" / "active" / ..).and(reverse_proxy_filter(
            "".to_string(),
            server_stub.http_node_address(),
        ));

        let block0_content = server_stub.block0();

        let block0 = warp::path!("block0").map(move || block0_content.clone());

        root.and(
            proposals
                .or(snapshot)
                .or(challenges)
                .or(fund)
                .or(account)
                .or(fragment)
                .or(message)
                .or(reviews)
                .or(settings)
                .or(explorer)
                .or(vote)
                .or(node)
                .or(block0),
        )
    };

    let v1 = {
        let root = warp::path!("v1" / ..);

        let fragments = warp::path!("fragments" / ..).and(reverse_proxy_filter(
            "".to_string(),
            server_stub.http_node_address(),
        ));

        let votes = warp::path!("votes" / ..).and(reverse_proxy_filter(
            "".to_string(),
            server_stub.http_node_address(),
        ));
        root.and(fragments.or(votes))
    };

    let vit_version = warp::path!("vit-version").and(reverse_proxy_filter(
        "".to_string(),
        server_stub.http_vit_address(),
    ));

    let health = warp::path!("health")
        .and(warp::get())
        .map(|| StatusCode::OK);

    let app = api.and(v0.or(v1).or(vit_version).or(health));

    match server_stub.protocol() {
        Protocol::Https(certs) => {
            warp::serve(app)
                .tls()
                .cert_path(&certs.cert_path)
                .key_path(&certs.key_path)
                .run(server_stub.base_address())
                .await;
        }
        Protocol::Http => {
            warp::serve(app).run(server_stub.base_address()).await;
        }
    }
}
