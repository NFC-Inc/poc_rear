use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Form, Json, Router,
};
use mongodb::{bson::oid::ObjectId, Client};
use tokio_stream::StreamExt;

use crate::{
    config::Config,
    wotd_models::{CreateWotdDto, DisplayWotdDto},
};

pub fn wotd_router() -> Router {
    Router::new()
        .route("/wotd", post(create_wotd))
        .route("/wotd/:word", get(get_one_wotd))
        .route("/wotd", get(get_wotd))
}

async fn get_one_wotd(client: Extension<std::sync::Arc<Client>>, word: Path<String>) -> Response {
    let collection: mongodb::Collection<DisplayWotdDto> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_WOTD);
    let word_name = word.to_string();

    log::info!("word: {word_name}");

    match collection
        .find_one(mongodb::bson::doc! { "word": &word_name }, None)
        .await
    {
        Ok(Some(wotd)) => {
            let id = wotd._id;
            tracing::info!("found wotd with id: {id}");
            log::info!("word: {:#?}", wotd);
            return (StatusCode::OK, Json(Some(wotd))).into_response();
        }
        Ok(None) => {
            tracing::warn!("no word found for: {}", word_name.to_string());
            return StatusCode::NOT_FOUND.into_response();
        }
        Err(err) => {
            tracing::error!(
                "server errored when trying to find: {}, {err}",
                word_name.to_string()
            );
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
}

async fn get_wotd(client: Extension<std::sync::Arc<Client>>) -> Response {
    let collection: mongodb::Collection<DisplayWotdDto> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_WOTD);
    match collection.find(None, None).await {
        Ok(mut cursor_wotd) => {
            let mut wotds = Vec::new();
            while let Some(wotd) = cursor_wotd.next().await {
                match wotd {
                    Ok(w) => wotds.push(w),
                    Err(err) => {
                        tracing::warn!("error occured during mongo currsor iteration: {err}")
                    }
                }
            }
            return (StatusCode::OK, Json(wotds)).into_response();
        }
        Err(err) => {
            tracing::error!("server errored when trying to find wotds: {err}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
}

async fn create_wotd(
    client: Extension<std::sync::Arc<Client>>,
    word_form: Form<CreateWotdDto>,
) -> (StatusCode, String) {
    let create_word = word_form;

    let wotd = DisplayWotdDto {
        _id: ObjectId::new(),
        created_by_id: ObjectId::new(),
        word: create_word.word.clone(),
        definition: create_word.definition.clone(),
        sentence: create_word.sentence.clone(),
    };

    let collection = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_WOTD);

    let result = collection.insert_one(wotd, None).await;

    match result {
        Ok(_) => (StatusCode::OK, "wotd added!".to_string()),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}
