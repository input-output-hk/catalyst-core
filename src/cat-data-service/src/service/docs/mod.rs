mod stoplight_elements;
use poem::{Route, get};

use crate::service::api::api;

pub fn docs() -> Route {
    let api_service = api();
    let spec = api_service.spec();

    let swagger_ui = api_service.swagger_ui();
    let rapidoc_ui = api_service.rapidoc();
    let redoc_ui = api_service.redoc();
    let openapi_explorer = api_service.openapi_explorer();
    let stoplight_ui = stoplight_elements::create_endpoint(&spec);

    Route::new().at("/", get(stoplight_ui))
    .nest("/swagger_ui", swagger_ui)
    .nest("/redoc", redoc_ui)
    .nest("/rapidoc", rapidoc_ui)
    .nest("/openapi_explorer", openapi_explorer)
    .at(
        "/cat-data-service.json",
        poem::endpoint::make_sync(move |_| spec.clone()),
    )

}
