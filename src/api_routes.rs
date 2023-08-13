use std::collections::HashMap;

use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Extension, Form, Json, Router,
};
use mongodb::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{config::Config, error::AxumHelloError};

pub fn api_router() -> Router {
    Router::new()
        .route("/users", post(create_user))
        .route("/users/:id", get(get_user))
        .route("/wotd/:word", get(get_wotd))
        .route("/wotd", post(create_wotd))
}

async fn get_wotd(
    client: Extension<std::sync::Arc<Client>>,
    word: Path<String>,
) -> (StatusCode, Json<Option<DisplayWotdDto>>) {
    let word_name = word.to_string();

    log::info!("word: {word_name}");

    let collection: mongodb::Collection<DisplayWotdDto> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_WOTD);
    match collection
        .find_one(mongodb::bson::doc! { "word": &word_name }, None)
        .await
    {
        Ok(Some(wotd)) => {
            let id = wotd.id;
            tracing::info!(target: "testing_events", tester = "test", "found wotd with id: {id}");
            return (StatusCode::OK, Json(Some(wotd)));
        }
        Ok(None) => {
            tracing::warn!("no word found for: {}", word_name.to_string());
            (StatusCode::NOT_FOUND, Json(None))
        }
        Err(err) => {
            tracing::error!(
                "server errored when trying to find: {}, {err}",
                word_name.to_string()
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(None))
        }
    }
}

async fn create_wotd(
    client: Extension<std::sync::Arc<Client>>,
    word_form: Form<CreateWotdDto>,
) -> (StatusCode, String) {
    let create_word = word_form;

    let wotd = DisplayWotdDto {
        id: Uuid::new_v4(),
        created_by_id: Uuid::new_v4(),
        word: create_word.word.clone(),
        definition: create_word.definition.clone(),
        sentence: create_word.sentence.clone(),
    };

    let collection = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_WOTD);

    let result = collection.insert_one(wotd, None).await;

    match result {
        Ok(_) => (StatusCode::OK, "user added".to_string()),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}

#[derive(Serialize, Deserialize)]
struct CreateWotdDto {
    word: String,
    definition: String,
    sentence: String,
}

#[derive(Serialize, Deserialize)]
pub struct DisplayWotdDto {
    id: Uuid,
    created_by_id: Uuid,
    word: String,
    definition: String,
    sentence: String,
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
