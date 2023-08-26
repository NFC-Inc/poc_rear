use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use bson::{doc, oid::ObjectId};
use mongodb::Collection;

use crate::user_models::{DtoUserCreate, UserModel};

pub async fn create_new_user(
    collection: Collection<UserModel>,
    create_user_form: DtoUserCreate,
) -> Result<Response, StatusCode> {
    // First check if the username already exists, if it does return a 409 CONFLICT code.
    if collection
        .find_one(doc! { "username": create_user_form.username.clone()}, None)
        .await
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some()
    {
        return Err(StatusCode::CONFLICT);
    }

    let user = UserModel {
        _id: ObjectId::new(),
        username: create_user_form.username.clone(),
        password: create_user_form.password.clone(),
        email: create_user_form.email.clone(),
        created_at: chrono::Utc::now().into(),
        updated_at: chrono::Utc::now().into(),
    };

    collection
        .insert_one(user, None)
        .await
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::OK, "user created!".to_string()).into_response())
}

pub async fn get_one_user(
    collection: Collection<UserModel>,
    username: String,
) -> Result<Response, StatusCode> {
    let user = collection
        .find_one(
            mongodb::bson::doc! { "username": &username.to_string() },
            None,
        )
        .await
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let id = user._id;
    tracing::debug!("found user with id: {id}");

    Ok((StatusCode::OK, Json(Some(user))).into_response())
}
