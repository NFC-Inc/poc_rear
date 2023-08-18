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
) -> Response {
    let collection = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_USERS);

    match collection
        .find_one(doc! { "username": create_user_form.username.clone()}, None)
        .await
    {
        Ok(None) => {
            let user = UserModel {
                _id: ObjectId::new(),
                username: create_user_form.username.clone(),
                password: create_user_form.password.clone(),
                email: create_user_form.email.clone(),
                created_at: chrono::Utc::now().into(),
                updated_at: chrono::Utc::now().into(),
            };
            let result = collection.insert_one(user, None).await;

            match result {
                Ok(_) => (StatusCode::OK, "user created!".to_string()).into_response(),
                Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
            }
        }
        Ok(Some(_user)) => (StatusCode::CONFLICT, "username already taken").into_response(),
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }

    // First check if the username already exists, if it does return a 409 CONFLICT code.
}

pub async fn get_user(
    Extension(client): Extension<std::sync::Arc<Client>>,
    Path(username): Path<String>,
) -> Response {
    // insert your application logic here
    let collection: mongodb::Collection<UserModel> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_USERS);

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
            (StatusCode::OK, Json(Some(user))).into_response()
        }
        Ok(None) => {
            tracing::warn!("no user found for: {}", username.to_string());
            StatusCode::NOT_FOUND.into_response()
        }
        Err(err) => {
            tracing::error!(
                "server errored when trying to find: {}, {err}",
                username.to_string()
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
