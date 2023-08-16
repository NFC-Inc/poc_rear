use axum::{
    middleware,
    routing::{get, post},
    Extension, Router,
};
use poc_rear_api_lib::{auth_routes, webutil};
use poc_rear_config_lib::config;
use std::{net::SocketAddr, sync::Arc};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

#[tokio::main]
async fn main() {
    let config = config::Config::new();
    config::Config::init_otel();
    let client = Arc::new(config::Config::init_mongo().await);

    config.log_config_values(log::Level::Info);

    let app = Router::new()
        .route("/api/wotd", get(poc_rear_api_lib::wotd_routes::get_wotd))
        .route(
            "/api/wotd",
            post(poc_rear_api_lib::wotd_routes::create_wotd),
        )
        .route(
            "/api/wotd/:word",
            get(poc_rear_api_lib::wotd_routes::get_one_wotd),
        )
        .route("/auth/logout", get(auth_routes::user_logout))
        .route("/auth", get(auth_routes::user_auth))
        .route(
            "/api/users/:username",
            get(poc_rear_api_lib::user_routes::get_user),
        )
        .route_layer(middleware::from_fn(auth_routes::auth))
        .route(
            "/api/users",
            post(poc_rear_api_lib::user_routes::create_user),
        )
        .route("/auth/login", post(auth_routes::user_login))
        .route("/auth/login", get(auth_routes::get_user_login))
        .layer(Extension(client))
        .nest("/health", webutil::health_router())
        .fallback(webutil::not_found)
        .layer(
            TraceLayer::new_for_http()
                .on_request(trace::DefaultOnRequest::new().level(Level::INFO))
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let addr = SocketAddr::from((config.service_ip(), config.service_port()));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
