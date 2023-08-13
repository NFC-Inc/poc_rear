use axum::{Extension, Router};
use std::{net::SocketAddr, sync::Arc};
use tower_http::trace::TraceLayer;

mod api_routes;
mod config;
mod config_env;
mod error;
mod webutil;

#[tokio::main]
async fn main() {
    let config = config::Config::new();
    config::Config::init_otel();

    let client = Arc::new(config::Config::init_mongo::<api_routes::DisplayWotdDto>().await);

    config.log_config_values(log::Level::Info);

    let app = Router::new()
        .nest("/api", api_routes::api_router())
        .nest("/health", webutil::health_router())
        .layer(Extension(client))
        .layer(TraceLayer::new_for_http())
        .fallback(webutil::not_found);

    let addr = SocketAddr::from((config.service_ip(), config.service_port()));
    log::info!("listening on {}", addr);
    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app.into_make_service(),
    )
    .await
    .unwrap();
}
