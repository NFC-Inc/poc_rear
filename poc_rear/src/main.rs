use api_lib::{auth_routes, user_routes, webutil, word_routes};
use axum::{
    middleware,
    routing::{get, post},
    Extension, Router,
};
use config_lib::config;
use mongodb::Client;
use std::{net::SocketAddr, sync::Arc};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

#[tokio::main]
async fn main() {
    let config = config::Config::new();
    config::Config::init_otel();
    let client: Arc<Client> = Arc::new(config::Config::init_mongo().await);
    config.log_config_values(log::Level::Info);
    let app = Router::new()
        .route("/api/wotd", get(word_routes::get_wotd))
        .route("/api/wotd/update", post(word_routes::update_wotd))
        .route("/api/wotd/suggest", post(word_routes::suggest_new_wotd))
        .route("/api/words", post(word_routes::create_word))
        .route("/api/words", get(word_routes::get_words))
        .route("/api/words/:word", get(word_routes::get_word))
        .route("/api/users/:username", get(user_routes::get_user))
        .route("/auth/logout", get(auth_routes::user_logout))
        .route_layer(middleware::from_fn(auth_routes::auth)) // All routes above will require 'access_token' cookie
        .route("/auth/login", post(auth_routes::user_login))
        .route("/auth/account", post(user_routes::create_user))
        .layer(Extension(client))
        .nest("/health", webutil::health_router())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
        )
        .fallback(webutil::not_found);

    let addr = SocketAddr::from((config.service_ip(), config.service_port()));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
