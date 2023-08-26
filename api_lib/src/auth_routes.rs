use anyhow::Result;
use config_lib::{config::Config, config_env::ConfigEnvKey};
use std::sync::Arc;
use user_lib::user_models::{DtoUser, DtoUserLogin, UserModel};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Form,
};
use mongodb::{bson::doc, Client, Collection};

pub async fn user_login(
    Extension(client): Extension<Arc<Client>>,
    Form(user_form): Form<DtoUserLogin>,
) -> Result<Response, StatusCode> {
    let username = user_form.username.clone();
    let password = user_form.password.clone();

    tracing::info!("username: {username}, password: {password}");

    let user_collection: Collection<UserModel> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_USERS);

    let user = user_collection
        .find_one(doc! { "username": &username }, None)
        .await
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if user.username == username && user.password == password {
        tracing::info!(user.username, "matched user!");
        build_login_response(username)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

fn build_login_response(username: String) -> Result<Response, StatusCode> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(
            "Set-Cookie",
            format!(
                "{}={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=999999{}",
                Config::AUTH_TOKEN_STRING,
                format!("testing.{}.testing", username),
                if !bool::from(ConfigEnvKey::DevMode) {
                    "; Secure;"
                } else {
                    ""
                }
            ),
        )
        .body(http_body::Empty::new())
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_response())
}

fn build_logout_response() -> Result<Response, StatusCode> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(
            "Set-Cookie",
            format!(
                "{}={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=999999{}",
                Config::AUTH_TOKEN_STRING,
                "invalidated",
                if !bool::from(ConfigEnvKey::DevMode) {
                    "; Secure;"
                } else {
                    ""
                }
            ),
        )
        .body(http_body::Empty::new())
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_response())
}

pub async fn user_logout(
    Extension(client): Extension<Arc<Client>>,
    Extension(user): Extension<DtoUser>,
) -> Result<Response, StatusCode> {
    let user_collection: Collection<UserModel> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_USERS);

    user_collection
        .find_one(doc! { "username": &user.username }, None)
        .await
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    build_logout_response()
}
