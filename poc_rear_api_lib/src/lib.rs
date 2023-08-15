use axum::Router;

pub mod auth_routes;
pub mod user_routes;
pub mod webutil;
pub mod wotd_routes;

pub fn auth_routes() -> Router {
    Router::new().nest("/user", user_routes::user_router())
}

pub fn api_routes() -> Router {
    Router::new()
        .nest("/user", user_routes::user_router())
        .nest("/wotd", wotd_routes::wotd_router())
}
