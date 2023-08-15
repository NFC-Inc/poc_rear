use anyhow::Result;
use std::sync::Arc;

use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Extension, Form,
};
use mongodb::{bson::doc, Client, Collection};

use crate::{
    config::Config,
    user_models::{User, UserLogin},
};

pub async fn user_login(
    Extension(client): Extension<Arc<Client>>,
    Form(user_form): Form<UserLogin>,
) -> Result<Response, StatusCode> {
    let username = user_form.username.clone();
    let password = user_form.password.clone();

    let user_collection: Collection<User> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_USER);
    match user_collection
        .find_one(doc! { "username": &username }, None)
        .await
    {
        Ok(Some(u)) => {
            if u.username == username && u.password == password {
                log::info!("matched user!");
                return Ok(build_login_response(username));
            }

            // User password did not match.
            return Err(StatusCode::BAD_REQUEST);
        }
        Ok(None) => {
            // User was not found in the database.
            return Err(StatusCode::BAD_REQUEST);
        }
        Err(err) => {
            // An error occurred while searching the database.
            tracing::error!("an error occurred while searching for username ({username}): {err}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}

pub async fn get_user_login() -> Result<Response, StatusCode> {
    return Ok(build_login_response("temp_testing".to_string()));
}

fn build_login_response(username: String) -> Response {
    return Response::builder()
        .status(StatusCode::OK)
        .header(
            "Set-Cookie",
            format!(
                "{}={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=999999{}",
                Config::AUTH_TOKEN_STRING,
                format!("testing.{}.testing", username),
                if !Config::DEVELOPMENT {
                    "; Secure;"
                } else {
                    ""
                }
            ),
        )
        .body(http_body::Empty::new())
        .unwrap()
        .into_response();
}

pub async fn user_logout() -> Result<Response, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
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
                let parts: Vec<&str> = jwt_access_token.split(".").collect();
                let user_collection: Collection<User> = client
                    .database(Config::MONGO_DB_NAME)
                    .collection(Config::MONGO_COLL_NAME_USER);

                match user_collection
                    .find_one(doc! {"username": parts.get(1)}, None)
                    .await
                {
                    Ok(Some(user)) => {
                        log::info!("found user: {user:#?}");
                        req.extensions_mut().insert(user);
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
