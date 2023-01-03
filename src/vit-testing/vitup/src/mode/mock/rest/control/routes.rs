use crate::mode::mock::ContextLock;
use jortestkit::web::api_token::API_TOKEN_HEADER;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

use super::handlers::*;

pub async fn control_filter(
    context: ContextLock,
    root: BoxedFilter<()>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let working_dir = context.read().unwrap().working_dir();
    let is_token_enabled = context.read().unwrap().api_token().is_some();

    let with_context = warp::any().map(move || context.clone());

    let api_token_filter = if is_token_enabled {
        warp::header::header(API_TOKEN_HEADER)
            .and(with_context.clone())
            .and_then(authorize_token)
            .and(warp::any())
            .untuple_one()
            .boxed()
    } else {
        warp::any().boxed()
    };

    let files = {
        let root = warp::path!("files" / ..).boxed();

        let get = warp::path("get").and(warp::fs::dir(working_dir));
        let list = warp::path!("list")
            .and(warp::get())
            .and(with_context.clone())
            .and_then(file_lister_handler);

        root.and(get.or(list)).boxed()
    };

    let command = {
        let root = warp::path!("command" / ..);

        let reset = warp::path!("reset")
            .and(warp::post())
            .and(with_context.clone())
            .and(warp::body::json())
            .and_then(command_reset_mock);

        let availability = warp::path!("available" / bool)
            .and(warp::post())
            .and(with_context.clone())
            .and_then(command_available);

        let set_error_code = warp::path!("error-code" / u16)
            .and(warp::post())
            .and(with_context.clone())
            .and_then(command_error_code);

        let fund = {
            let root = warp::path!("fund");

            let fund_id = warp::path!("id" / i32)
                .and(warp::post())
                .and(with_context.clone())
                .and_then(command_fund_id);

            let fund_update = warp::path!("update")
                .and(warp::put())
                .and(warp::body::json())
                .and(with_context.clone())
                .and_then(command_update_fund);

            root.and(fund_id.or(fund_update))
        };

        let version = warp::path!("version" / String)
            .and(warp::post())
            .and(with_context.clone())
            .and_then(command_version);

        let block_account = {
            let root = warp::path!("block-account" / ..);

            let block_counter = warp::path!(u32)
                .and(warp::post())
                .and(with_context.clone())
                .and_then(command_block_account);

            let reset = warp::path!("reset")
                .and(warp::post())
                .and(with_context.clone())
                .and_then(command_reset_block_account);

            root.and(block_counter.or(reset)).boxed()
        };

        let fragment_strategy = {
            let root = warp::path!("fragments" / ..);

            let reject = warp::path!("reject")
                .and(warp::post())
                .and(with_context.clone())
                .and_then(command_reject);

            let accept = warp::path!("accept")
                .and(warp::post())
                .and(with_context.clone())
                .and_then(command_accept);

            let pending = warp::path!("pending")
                .and(warp::post())
                .and(with_context.clone())
                .and_then(command_pending);

            let reset = warp::path!("reset")
                .and(warp::post())
                .and(with_context.clone())
                .and_then(command_reset);

            let forget = warp::path!("forget")
                .and(warp::post())
                .and(with_context.clone())
                .and_then(command_forget);

            let update = {
                let root = warp::path!("update" / ..);

                let reject = warp::path!(String / "reject")
                    .and(warp::post())
                    .and(with_context.clone())
                    .and_then(command_update_reject);

                let accept = warp::path!(String / "accept")
                    .and(warp::post())
                    .and(with_context.clone())
                    .and_then(command_update_accept);

                let pending = warp::path!(String / "pending")
                    .and(warp::post())
                    .and(with_context.clone())
                    .and_then(command_update_pending);

                let forget = warp::path!(String / "forget")
                    .and(warp::post())
                    .and(with_context.clone())
                    .and_then(command_update_forget);

                root.and(reject.or(accept).or(pending).or(forget)).boxed()
            };

            root.and(
                reject
                    .or(accept)
                    .or(pending)
                    .or(reset)
                    .or(update)
                    .or(forget),
            )
            .boxed()
        };

        let network_strategy = {
            let root = warp::path!("congestion" / ..);

            let normal = warp::path!("normal")
                .and(warp::post())
                .and(with_context.clone())
                .and_then(command_congestion_normal);

            let jammed = warp::path!("jammed")
                .and(warp::post())
                .and(with_context.clone())
                .and_then(command_congestion_jammed);

            let moderate = warp::path!("moderate")
                .and(warp::post())
                .and(with_context.clone())
                .and_then(command_congestion_moderate);

            let reset = warp::path!("reset")
                .and(warp::post())
                .and(with_context.clone())
                .and_then(command_congestion_reset);

            root.and(normal.or(jammed).or(moderate).or(reset)).boxed()
        };

        let snapshot_service = {
            let root = warp::path!("snapshot" / ..);

            let create = warp::path!("create")
                .and(warp::post())
                .and(warp::body::json())
                .and(with_context)
                .and_then(command_create_snapshot);

            root.and(create).boxed()
        };

        root.and(
            reset
                .or(availability)
                .or(set_error_code)
                .or(fund)
                .or(block_account)
                .or(fragment_strategy)
                .or(network_strategy)
                .or(version)
                .or(snapshot_service),
        )
        .boxed()
    };
    root.and(api_token_filter).and(command.or(files))
}
