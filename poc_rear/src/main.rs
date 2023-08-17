use axum::{
    middleware,
    routing::{get, post},
    Extension, Router,
};
use mongodb::Client;
use api_lib::{auth_routes, user_routes, webutil, word_routes};
use config_lib::config;
use std::{
    net::SocketAddr,
    sync::Arc,
};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

fn wotd_routes() -> Router {
    Router::new()
        .route("/", get(word_routes::get_wotd))
        .route("/suggest", post(word_routes::suggest_new_wotd))
        .route("/update", post(word_routes::update_wotd))
}

fn words_routes() -> Router {
    Router::new()
        .route("/", post(word_routes::create_word))
        .route("/", get(word_routes::get_words))
        .route("/:word", get(word_routes::get_word))
}

fn users_routes() -> Router {
    Router::new().route("/:username", get(user_routes::get_user))
}

#[tokio::main]
async fn main() {
    let config = config::Config::new();
    config::Config::init_otel();
    let client: Arc<Client> = Arc::new(config::Config::init_mongo().await);

    config.log_config_values(log::Level::Info);

    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .nest("/wotd", wotd_routes())
                .nest("/words", words_routes())
                .nest("/users", users_routes()),
        )
        .route_layer(middleware::from_fn(auth_routes::auth))
        .nest(
            "/auth",
            Router::new()
                .route("/logout", get(auth_routes::user_logout))
                .route("/login", post(auth_routes::user_login))
                .route("/account", post(user_routes::create_user)),
        )
        .layer(Extension(client))
        .nest("/health", webutil::health_router())
        .layer(
            TraceLayer::new_for_http()
                .on_request(trace::DefaultOnRequest::new().level(Level::INFO))
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .fallback(webutil::not_found);

    let addr = SocketAddr::from((config.service_ip(), config.service_port()));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
