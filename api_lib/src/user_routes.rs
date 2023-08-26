use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Form, Json,
};
use bson::doc;
use config_lib::config::Config;
use mongodb::{bson::oid::ObjectId, Client};
use user_lib::user_models::{DtoUserCreate, UserModel};

pub async fn create_user(
    Extension(client): Extension<std::sync::Arc<Client>>,
    Form(create_user_form): Form<DtoUserCreate>,
) -> Result<Response, StatusCode> {
    let collection = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_USERS);

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

pub async fn get_user(
    Extension(client): Extension<std::sync::Arc<Client>>,
    Path(username): Path<String>,
) -> Result<Response, StatusCode> {
    // insert your application logic here
    let collection: mongodb::Collection<UserModel> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_USERS);

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
