use axum::{http::StatusCode, routing::get, Json, Router};
use serde::Serialize;

#[derive(Serialize)]
pub struct Response {
    pub message: String,
}

pub fn health_router() -> Router {
    Router::new()
        .route("/readiness", get(healthcheck_readiness))
        .route("/liveness", get(healthcheck_liveness))
}

pub async fn healthcheck_readiness() -> (StatusCode, Json<Response>) {
    let response = Response {
        message: "Everything is working fine!".to_string(),
    };
    (StatusCode::OK, Json(response))
}

pub async fn healthcheck_liveness() -> (StatusCode, Json<Response>) {
    let response = Response {
        message: "Everything is working fine!".to_string(),
    };
    (StatusCode::OK, Json(response))
}
