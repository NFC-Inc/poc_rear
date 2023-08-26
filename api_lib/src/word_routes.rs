use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Form, Json,
};
use config_lib::config::Config;
use mongodb::{bson::oid::ObjectId, Client};
use tokio_stream::StreamExt;
use user_lib::user_models::DtoUser;
use wotd_lib::{
    word_models::{DtoWotdCreate, WordModel},
    word_queue::QueueItemWordModel,
};

pub async fn suggest_new_wotd(
    Extension(dto_user): Extension<DtoUser>,
    Extension(client): Extension<std::sync::Arc<Client>>,
    Form(dto_word_suggestion): Form<DtoWotdCreate>,
) -> Result<Response, StatusCode> {
    let queue_collection: mongodb::Collection<QueueItemWordModel> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_QUEUE_WORDS);

    match queue_collection
        .find_one(
            mongodb::bson::doc! {"word.word": dto_word_suggestion.word.clone() },
            None,
        )
        .await
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        Some(word) => Ok((
            StatusCode::BAD_REQUEST,
            format!(
                "{} word has already been suggested, and is in the queue!",
                word.word.word
            ),
        )
            .into_response()),
        None => {
            let words_collection: mongodb::Collection<WordModel> = client
                .database(Config::MONGO_DB_NAME)
                .collection(Config::MONGO_COLL_NAME_WORDS);

            let suggested_word = match words_collection
                .find_one(
                    mongodb::bson::doc! {"word": dto_word_suggestion.word.clone() },
                    None,
                )
                .await
                .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?
            {
                Some(word) => {
                    tracing::info!("found existing word!");
                    word
                }
                None => {
                    let new_word = WordModel {
                        _id: ObjectId::new(),
                        created_by_id: dto_user._id,
                        word: dto_word_suggestion.word.clone(),
                        definition: dto_word_suggestion.definition,
                        sentence: dto_word_suggestion.sentence,
                        created_at: chrono::Utc::now().into(),
                        updated_at: chrono::Utc::now().into(),
                    };

                    tracing::info!("creating new word!");

                    if let Err(_err) = words_collection.insert_one(new_word.clone(), None).await {
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }

                    new_word
                }
            };

            let suggestion = QueueItemWordModel {
                _id: ObjectId::new(),
                added_at: chrono::Utc::now().into(),
                word: suggested_word,
            };

            queue_collection
                .insert_one(suggestion, None)
                .await
                .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?;

            return Ok((StatusCode::OK, "wotd added!".to_string()).into_response());
        }
    }
}

pub async fn get_wotd(
    Extension(_user): Extension<DtoUser>,
    Extension(client): Extension<std::sync::Arc<Client>>,
) -> Result<Response, StatusCode> {
    let collection: mongodb::Collection<QueueItemWordModel> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_QUEUE_WORDS);

    let options = mongodb::options::FindOneOptions::builder()
        .sort(mongodb::bson::doc! {"added_at": 1})
        .build();

    match collection.find_one(mongodb::bson::doc! {}, options).await {
        Ok(Some(wotd)) => Ok((StatusCode::OK, Json(Some(wotd))).into_response()),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_err) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn update_wotd(
    Extension(_user): Extension<DtoUser>,
    Extension(client): Extension<std::sync::Arc<Client>>,
) -> Result<Response, StatusCode> {
    let collection: mongodb::Collection<QueueItemWordModel> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_QUEUE_WORDS);

    let options = mongodb::options::FindOneOptions::builder()
        .sort(mongodb::bson::doc! {"added_at": 1})
        .build();

    let wotd = collection
        .find_one(mongodb::bson::doc! {}, options)
        .await
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Delete the document
    let deleted = collection
        .delete_one(mongodb::bson::doc! {"_id": wotd._id}, None)
        .await
        .map_err(|_err| StatusCode::NOT_FOUND)?;

    tracing::debug!("Deleted {} document(s).", deleted.deleted_count);
    Ok((StatusCode::OK, Json(Some(wotd))).into_response())
}

pub async fn get_word(
    Extension(_user): Extension<DtoUser>,
    Extension(client): Extension<std::sync::Arc<Client>>,
    word: Path<String>,
) -> Result<Response, StatusCode> {
    let collection: mongodb::Collection<WordModel> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_WORDS);
    let word_name = word.to_string();

    let wotd = collection
        .find_one(mongodb::bson::doc! { "word": &word_name }, None)
        .await
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok((StatusCode::OK, Json(Some(wotd))).into_response())
}

pub async fn get_words(
    Extension(_user): Extension<DtoUser>,
    Extension(client): Extension<std::sync::Arc<Client>>,
) -> Result<Response, StatusCode> {
    let collection: mongodb::Collection<WordModel> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_WORDS);
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

pub async fn create_word(
    Extension(dto_user): Extension<DtoUser>,
    Extension(client): Extension<std::sync::Arc<Client>>,
    Form(create_word_dto): Form<DtoWotdCreate>,
) -> Result<Response, StatusCode> {
    let create_word = create_word_dto;

    let wotd = WordModel {
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
        .collection(Config::MONGO_COLL_NAME_WORDS);

    let _result = collection
        .insert_one(wotd, None)
        .await
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::OK, "wotd added!".to_string()).into_response())
}
