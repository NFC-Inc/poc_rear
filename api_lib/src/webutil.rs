use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct CustomResponse {
    pub message: String,
}

pub fn health_router() -> Router {
    Router::new()
        .route("/readiness", get(healthcheck_readiness))
        .route("/liveness", get(healthcheck_liveness))
}

pub async fn healthcheck_readiness() -> Result<Response, StatusCode> {
    Ok((
        StatusCode::OK,
        Json(CustomResponse {
            message: "Everything is working fine!".to_string(),
        }),
    )
        .into_response())
}

pub async fn healthcheck_liveness() -> Result<Response, StatusCode> {
    Ok((
        StatusCode::OK,
        Json(CustomResponse {
            message: "Everything is working fine!".to_string(),
        }),
    )
        .into_response())
}

pub async fn not_found() -> Result<Response, StatusCode> {
    Ok((
        StatusCode::NOT_FOUND,
        Json(CustomResponse {
            message: "Resource not found".to_string(),
        }),
    )
        .into_response())
}
