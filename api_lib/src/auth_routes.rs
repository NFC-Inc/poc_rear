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
) -> Response {
    let username = user_form.username.clone();
    let password = user_form.password.clone();

    tracing::info!("username: {username}, password: {password}");

    let user_collection: Collection<UserModel> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_USERS);
    match user_collection
        .find_one(doc! { "username": &username }, None)
        .await
    {
        Ok(Some(u)) => {
            if u.username == username && u.password == password {
                tracing::info!(u.username, "matched user!");
                return build_login_response(username);
            }
            StatusCode::NOT_FOUND.into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(err) => {
            // An error occurred while searching the database.
            tracing::error!("an error occurred while searching for username ({username}): {err}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn build_login_response(username: String) -> Response {
    Response::builder()
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
        .unwrap()
        .into_response()
}

fn build_logout_response() -> Response {
    Response::builder()
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
        .unwrap()
        .into_response()
}

pub async fn user_logout(
    Extension(client): Extension<Arc<Client>>,
    Extension(user): Extension<DtoUser>,
) -> Result<Response, StatusCode> {
    let user_collection: Collection<UserModel> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_USERS);
    match user_collection
        .find_one(doc! { "username": &user.username }, None)
        .await
    {
        Ok(Some(_)) => Ok(build_logout_response()),
        Ok(None) => Err(StatusCode::BAD_REQUEST),
        Err(err) => {
            // An error occurred while searching the database.
            tracing::error!(
                "an error occurred while searching for username ({}): {err}",
                user.username
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
