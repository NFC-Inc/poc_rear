use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use bson::oid::ObjectId;
use mongodb::Collection;
use tokio_stream::StreamExt;

use crate::word_models::{DtoWotdCreate, WordModel};

pub async fn get_one_word(
    collection: Collection<WordModel>,
    word: String,
) -> Result<Response, StatusCode> {
    let word_name = word.to_string();

    let wotd = collection
        .find_one(mongodb::bson::doc! { "word": &word_name }, None)
        .await
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok((StatusCode::OK, Json(Some(wotd))).into_response())
}

pub async fn get_all_words(collection: Collection<WordModel>) -> Result<Response, StatusCode> {
    let mut cursor_wotd = collection
        .find(None, None)
        .await
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut wotds = Vec::new();
    while let Some(wotd) = cursor_wotd.next().await {
        match wotd {
            Ok(w) => wotds.push(w),
            Err(err) => {
                tracing::warn!("error occured during mongo cursor iteration: {err}")
            }
        }
    }
    Ok((StatusCode::OK, Json(wotds)).into_response())
}

pub async fn create_one_word(
    collection: Collection<WordModel>,
    user_id: ObjectId,
    create_word_dto: DtoWotdCreate,
) -> Result<Response, StatusCode> {
    let create_word = create_word_dto;

    let wotd = WordModel {
        _id: ObjectId::new(),
        created_by_id: user_id,
        word: create_word.word.clone(),
        definition: create_word.definition.clone(),
        sentence: create_word.sentence.clone(),
        created_at: chrono::Utc::now().into(),
        updated_at: chrono::Utc::now().into(),
    };

    let _result = collection
        .insert_one(wotd, None)
        .await
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::OK, "wotd added!".to_string()).into_response())
}
