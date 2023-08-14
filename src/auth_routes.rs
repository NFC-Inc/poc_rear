use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Form,
};
use mongodb::bson::oid::ObjectId;

use crate::{
    config::Config,
    user_models::{User, UserLogin},
};

pub async fn user_login(user_form: Form<UserLogin>) -> Result<Response, StatusCode> {
    log::info!("form: {user_form:#?}");
    let username = user_form.username.clone();
    let password = user_form.password.clone();

    if username == "davidular" && password == "admin" {
        return Ok(build_login_response());
    }

    Err(StatusCode::BAD_REQUEST)
}

pub async fn get_user_login() -> Result<Response, StatusCode> {
    return Ok(build_login_response());
}

fn build_login_response() -> Response {
    return Response::builder()
        .status(StatusCode::OK)
        .header(
            "Set-Cookie",
            format!(
                "access_token={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=999999{}",
                "testing.testing.testing",
                if !Config::DEVELOPMENT {
                    "; Secure;"
                } else {
                    ""
                }
            ),
            // SameSite=Strict; Secure; HttpOnly;
        )
        .body(http_body::Empty::new())
        .unwrap()
        .into_response();
}

pub async fn user_logout() -> Result<Response, StatusCode> {
    Err(StatusCode::UNAUTHORIZED)
}

pub async fn user_auth() -> Result<Response, StatusCode> {
    Err(StatusCode::UNAUTHORIZED)
}

pub async fn auth<T>(mut req: Request<T>, next: Next<T>) -> Result<Response, StatusCode> {
    log::info!("checking cookies");
    if let Some(cookie_header) = req.headers().get(http::header::COOKIE) {
        let cookies: Vec<_> = cookie_header.to_str().unwrap().split(';').collect();
        for cookie in cookies {
            log::info!("cookie: {}", cookie.trim());
        }
    }

    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());
    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    log::info!("header: {}", auth_header);

    if let Some(current_user) = authorize_user(auth_header).await {
        req.extensions_mut().insert(current_user);
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn authorize_user(auth_token: &str) -> Option<User> {
    if auth_token == "Bearer admin" {
        return Some(User {
            _id: ObjectId::new(),
            username: "admin".to_string(),
            password: "admin".to_string(),
        });
    }
    return None;
}