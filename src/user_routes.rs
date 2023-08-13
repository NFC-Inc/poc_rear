use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Form, Json, Router,
};
use mongodb::Client;

use crate::{
    config::Config,
    user_models::{CreateUser, User},
};

pub fn user_router() -> Router {
    Router::new()
        .route("/users", post(create_user))
        .route("/users/:username", get(get_user))
}

async fn create_user(
    client: Extension<std::sync::Arc<Client>>,
    user_form: Form<CreateUser>,
) -> Response {
    let create_user = user_form;

    let user = CreateUser {
        username: create_user.username.clone(),
    };

    let collection = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_USER);

    let result = collection.insert_one(user, None).await;

    match result {
        Ok(_) => (StatusCode::OK, "user created!".to_string()).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

async fn get_user(client: Extension<std::sync::Arc<Client>>, username: Path<String>) -> Response {
    // insert your application logic here
    let collection: mongodb::Collection<User> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_USER);

    match collection
        .find_one(
            mongodb::bson::doc! { "username": &username.to_string() },
            None,
        )
        .await
    {
        Ok(Some(user)) => {
            let id = user._id;
            tracing::info!("found user with id: {id}");
            return (StatusCode::OK, Json(Some(user))).into_response();
        }
        Ok(None) => {
            tracing::warn!("no user found for: {}", username.to_string());
            return StatusCode::NOT_FOUND.into_response();
        }
        Err(err) => {
            tracing::error!(
                "server errored when trying to find: {}, {err}",
                username.to_string()
            );
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
}
