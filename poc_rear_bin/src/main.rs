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
        // Get word of the day.
        .route("/api/wotd", get(poc_rear_api_lib::word_routes::get_wotd))
        .route(
            "/api/wotd/suggest",
            post(poc_rear_api_lib::word_routes::suggest_new_wotd),
        )
        .route(
            "/api/wotd/update",
            post(poc_rear_api_lib::word_routes::update_wotd),
        )
        // Get all words.
        .route("/api/word", get(poc_rear_api_lib::word_routes::get_words))
        // Get one word.
        .route(
            "/api/word/:word",
            get(poc_rear_api_lib::word_routes::get_one_word),
        )
        // Create a word.
        .route(
            "/api/word",
            post(poc_rear_api_lib::word_routes::create_word),
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
