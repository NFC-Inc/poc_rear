use std::collections::HashMap;

use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::error::AxumHelloError;

pub fn api_router() -> Router {
    Router::new()
        .route("/users", post(create_user))
        .route("/users/:id", get(get_user))
}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    tracing::info!("created a default user!");
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

async fn get_user(
    Path(params): Path<HashMap<String, String>>,
) -> Result<Json<User>, AxumHelloError> {
    // insert your application logic here
    let id = params.get("id");
    match id {
        Some(i) => match i.parse::<u64>() {
            Ok(id) => {
                let user = User {
                    id,
                    username: "defaulted".to_string(),
                };
                tracing::info!("found default user!");
                Ok(Json(user))
            }
            Err(e) => {
                tracing::error!("error finding user with id: {:#?}!", id);
                Err(AxumHelloError::BadRequest(format!(
                    "failed when parsing id: {}",
                    e
                )))
            }
        },
        None => Err(AxumHelloError::BadRequest(
            "Missing ':id' in path params".to_string(),
        )),
    }
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
