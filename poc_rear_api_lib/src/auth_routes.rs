use anyhow::Result;
use poc_rear_config_lib::{config::Config, config_env::ConfigEnvKey};
use poc_rear_user_lib::user_models::{DtoUser, DtoUserLogin, UserModel};
use std::sync::Arc;

use axum::{
    http::{Request, StatusCode},
    middleware::Next,
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

    let user_collection: Collection<UserModel> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_USER);
    match user_collection
        .find_one(doc! { "username": &username }, None)
        .await
    {
        Ok(Some(u)) => {
            if u.username == username && u.password == password {
                tracing::info!(u.username, "matched user!");
                return Ok(build_login_response(username));
            }
            Err(StatusCode::BAD_REQUEST)
        }
        Ok(None) => Err(StatusCode::BAD_REQUEST),
        Err(err) => {
            // An error occurred while searching the database.
            tracing::error!("an error occurred while searching for username ({username}): {err}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_user_login() -> Result<Response, StatusCode> {
    Ok(build_login_response("temp_testing".to_string()))
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
        .collection(Config::MONGO_COLL_NAME_USER);
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

pub async fn user_auth() -> Result<Response, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn auth<T>(
    Extension(client): Extension<Arc<Client>>,
    mut req: Request<T>,
    next: Next<T>,
) -> Result<Response, StatusCode> {
    if let Some(cookie_header) = req.headers().get(http::header::COOKIE) {
        let cookies: Vec<_> = cookie_header.to_str().unwrap().split(';').collect();
        for cookie in cookies {
            if cookie.contains(Config::AUTH_TOKEN_STRING) {
                log::info!("found auth token: {cookie}");
                let jwt_access_token =
                    cookie.replace(&format!("{}=", Config::AUTH_TOKEN_STRING), "");
                let parts: Vec<&str> = jwt_access_token.split('.').collect();
                let user_collection: Collection<UserModel> = client
                    .database(Config::MONGO_DB_NAME)
                    .collection(Config::MONGO_COLL_NAME_USER);

                match user_collection
                    .find_one(doc! {"username": parts.get(1)}, None)
                    .await
                {
                    Ok(Some(user)) => {
                        let dto_user = DtoUser::from(user);
                        log::info!("found user: {dto_user:#?}");
                        req.extensions_mut().insert(dto_user);
                        return Ok(next.run(req).await);
                    }
                    Ok(None) => {
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                    Err(err) => {
                        tracing::error!("an error occurred while searching db: {err}");
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                }
            }
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}
