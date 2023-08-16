use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Form, Json,
};
use mongodb::{bson::oid::ObjectId, Client};
use poc_rear_config_lib::config::Config;
use poc_rear_user_lib::user_models::DtoUser;
use poc_rear_wotd_lib::{
    word_models::{DtoWotdCreate, WordModel},
    word_queue::QueueItemWordModel,
};
use tokio_stream::StreamExt;

pub async fn suggest_new_wotd(
    Extension(dto_user): Extension<DtoUser>,
    Extension(client): Extension<std::sync::Arc<Client>>,
    Form(dto_word_suggestion): Form<DtoWotdCreate>,
) -> Response {
    let queue_collection: mongodb::Collection<QueueItemWordModel> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_QUEUE_WORDS);

    match queue_collection
        .find_one(
            mongodb::bson::doc! {"word.word": dto_word_suggestion.word.clone() },
            None,
        )
        .await
    {
        Ok(Some(word)) => (
            StatusCode::BAD_REQUEST,
            format!(
                "{} word has already been suggested, and is in the queue!",
                word.word.word
            ),
        )
            .into_response(),
        Ok(None) => {
            let words_collection: mongodb::Collection<WordModel> = client
                .database(Config::MONGO_DB_NAME)
                .collection(Config::MONGO_COLL_NAME_WORDS);

            let suggested_word = match words_collection
                .find_one(
                    mongodb::bson::doc! {"word": dto_word_suggestion.word.clone() },
                    None,
                )
                .await
            {
                Ok(Some(word)) => {
                    tracing::info!("found existing word!");
                    word
                }
                Ok(None) => {
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
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }

                    new_word
                }
                Err(_err) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };

            let suggestion = QueueItemWordModel {
                _id: ObjectId::new(),
                added_at: chrono::Utc::now().into(),
                word: suggested_word,
            };

            match queue_collection.insert_one(suggestion, None).await {
                Ok(_) => (StatusCode::OK, "wotd added!".to_string()).into_response(),
                Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
            }
        }
        Err(_err) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn get_wotd(
    Extension(_user): Extension<DtoUser>,
    Extension(client): Extension<std::sync::Arc<Client>>,
) -> Response {
    let collection: mongodb::Collection<QueueItemWordModel> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_QUEUE_WORDS);

    let options = mongodb::options::FindOneOptions::builder()
        .sort(mongodb::bson::doc! {"added_at": 1})
        .build();

    match collection.find_one(mongodb::bson::doc! {}, options).await {
        Ok(Some(wotd)) => (StatusCode::OK, Json(Some(wotd))).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_err) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn update_wotd(
    Extension(_user): Extension<DtoUser>,
    Extension(client): Extension<std::sync::Arc<Client>>,
) -> Response {
    let collection: mongodb::Collection<QueueItemWordModel> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_QUEUE_WORDS);

    let options = mongodb::options::FindOneOptions::builder()
        .sort(mongodb::bson::doc! {"added_at": 1})
        .build();

    match collection.find_one(mongodb::bson::doc! {}, options).await {
        Ok(Some(wotd)) => {
            // Delete the document
            match collection
                .delete_one(mongodb::bson::doc! {"_id": wotd._id}, None)
                .await
            {
                Ok(deleted) => {
                    println!("Deleted {} document(s).", deleted.deleted_count);
                    return (StatusCode::OK, Json(Some(wotd))).into_response();
                }
                Err(_) => return StatusCode::NOT_FOUND.into_response(),
            }
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_err) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn get_one_word(
    Extension(_user): Extension<DtoUser>,
    Extension(client): Extension<std::sync::Arc<Client>>,
    word: Path<String>,
) -> Response {
    let collection: mongodb::Collection<WordModel> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_WORDS);
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

pub async fn get_words(
    Extension(_user): Extension<DtoUser>,
    Extension(client): Extension<std::sync::Arc<Client>>,
) -> Response {
    let collection: mongodb::Collection<WordModel> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_WORDS);
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

pub async fn create_word(
    Extension(dto_user): Extension<DtoUser>,
    Extension(client): Extension<std::sync::Arc<Client>>,
    Form(create_word_dto): Form<DtoWotdCreate>,
) -> (StatusCode, String) {
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

    let result = collection.insert_one(wotd, None).await;

    match result {
        Ok(_) => (StatusCode::OK, "wotd added!".to_string()),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}
