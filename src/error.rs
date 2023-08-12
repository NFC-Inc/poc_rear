use axum::{http::StatusCode, response::IntoResponse};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum AxumHelloError {
    BadRequest(String),
}

impl IntoResponse for AxumHelloError {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        match self {
            Self::BadRequest(m) => axum::http::Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "text/plain")
                .body(axum::body::Body::from(m))
                .unwrap(),
        }
    }
}
