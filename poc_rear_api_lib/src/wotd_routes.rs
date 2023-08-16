use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Form, Json,
};
use mongodb::{bson::oid::ObjectId, Client};
use poc_rear_config_lib::config::Config;
use poc_rear_user_lib::user_models::DtoUser;
use poc_rear_wotd_lib::wotd_models::{DtoWotdCreate, WotdModel};
use tokio_stream::StreamExt;

pub async fn get_one_wotd(
    Extension(_user): Extension<DtoUser>,
    Extension(client): Extension<std::sync::Arc<Client>>,
    word: Path<String>,
) -> Response {
    let collection: mongodb::Collection<WotdModel> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_WOTD);
    let word_name = word.to_string();

    match collection
        .find_one(mongodb::bson::doc! { "word": &word_name }, None)
        .await
    {
        Ok(Some(wotd)) => (StatusCode::OK, Json(Some(wotd))).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(err) => {
            tracing::error!(
                "server errored when trying to find: {}, {err}",
                word_name.to_string()
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn get_wotd(
    Extension(_user): Extension<DtoUser>,
    Extension(client): Extension<std::sync::Arc<Client>>,
) -> Response {
    let collection: mongodb::Collection<WotdModel> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_WOTD);
    match collection.find(None, None).await {
        Ok(mut cursor_wotd) => {
            let mut wotds = Vec::new();
            while let Some(wotd) = cursor_wotd.next().await {
                match wotd {
                    Ok(w) => wotds.push(w),
                    Err(err) => {
                        tracing::warn!("error occured during mongo cursor iteration: {err}")
                    }
                }
            }
            (StatusCode::OK, Json(wotds)).into_response()
        }
        Err(err) => {
            tracing::error!("server errored when trying to find wotds: {err}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn create_wotd(
    Extension(dto_user): Extension<DtoUser>,
    Extension(client): Extension<std::sync::Arc<Client>>,
    Form(create_word_dto): Form<DtoWotdCreate>,
) -> (StatusCode, String) {
    let create_word = create_word_dto;

    let wotd = WotdModel {
        _id: ObjectId::new(),
        created_by_id: dto_user._id,
        word: create_word.word.clone(),
        definition: create_word.definition.clone(),
        sentence: create_word.sentence.clone(),
        created_at: chrono::Utc::now().into(),
        updated_at: chrono::Utc::now().into(),
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
