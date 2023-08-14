use auth_routes::{auth, user_auth, user_login, user_logout};
use axum::{
    middleware,
    routing::{get, post},
    Extension, Router,
};
use std::{net::SocketAddr, sync::Arc};
use tower_http::trace::TraceLayer;

mod config;
mod config_env;
mod webutil;

mod wotd_models;
mod wotd_routes;

mod user_models;
mod user_routes;

mod auth_routes;

#[tokio::main]
async fn main() {
    let config = config::Config::new();
    config::Config::init_otel();

    let client = Arc::new(config::Config::init_mongo().await);

    config.log_config_values(log::Level::Info);

    let app = Router::new()
        .nest("/api", user_routes::user_router())
        .nest("/api", wotd_routes::wotd_router())
        .route("/auth/logout", get(user_logout))
        .route("/auth", get(user_auth))
        .route_layer(middleware::from_fn(auth))
        .route("/auth/login", post(user_login))
        .layer(Extension(client))
        .layer(TraceLayer::new_for_http())
        .nest("/health", webutil::health_router())
        .fallback(webutil::not_found);

    let addr = SocketAddr::from((config.service_ip(), config.service_port()));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
