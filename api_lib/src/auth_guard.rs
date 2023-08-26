use std::sync::Arc;

use alcoholic_jwt::{token_kid, validate, Validation, JWKS};
use anyhow::Result;
use bson::doc;
use config_lib::{config::Config, config_env::ConfigEnvKey};

use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Extension,
};
use mongodb::{Client, Collection};
use user_lib::user_models::{DtoUser, UserModel};

pub async fn auth<T>(
    Extension(client): Extension<Arc<Client>>,
    mut req: Request<T>,
    next: Next<T>,
) -> Result<Response, StatusCode> {
    let access_token = extract_access_token(&req)?;

    if !bool::from(ConfigEnvKey::DevMode) {
        validate_access_token(&access_token).await?;
    } else {
        let parts: Vec<&str> = access_token.split('.').collect();
        let user_collection: Collection<UserModel> = client
            .database(Config::MONGO_DB_NAME)
            .collection(Config::MONGO_COLL_NAME_USERS);

        let found_user = user_collection
            .find_one(doc! {"username": parts.get(1)}, None)
            .await
            .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let dto_user = DtoUser::from(found_user);
        tracing::info!("found user: {dto_user:#?}");
        req.extensions_mut().insert(dto_user);
    }

    Ok(next.run(req).await)
}

pub async fn validate_access_token(access_token: &str) -> Result<bool, StatusCode> {
    let authority = String::from(ConfigEnvKey::Authority);
    let uri = format!("{}{}", authority.as_str(), ".well-known/jwks.json");
    let jwks = fetch_jwks(&uri)
        .await
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?;
    tracing::trace!("got jwks from authority");

    let kid = token_kid(access_token)
        .map_err(|_err| StatusCode::UNAUTHORIZED)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let jwk = jwks.find(&kid).ok_or(StatusCode::UNAUTHORIZED)?;

    let validations = vec![Validation::Issuer(authority), Validation::SubjectPresent];
    validate(access_token, jwk, validations).map_err(|_err| StatusCode::UNAUTHORIZED)?;

    tracing::trace!("validated token");
    Ok(true)
}

async fn fetch_jwks(uri: &str) -> Result<JWKS, StatusCode> {
    match reqwest::get(uri).await {
        Ok(r) => {
            r
                .json::<JWKS>()
                .await
                .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_err) => {
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

fn extract_access_token<T>(req: &Request<T>) -> Result<String, StatusCode> {
    if let Some(cookie_header) = req.headers().get(http::header::COOKIE) {
        let cookies: Vec<_> = cookie_header.to_str().unwrap().split(';').collect();
        for cookie in cookies {
            if cookie.contains(Config::AUTH_TOKEN_STRING) {
                let jwt_access_token =
                    cookie.replace(&format!("{}=", Config::AUTH_TOKEN_STRING), "");
                tracing::trace!("extracted jwt from headers");
                return Ok(jwt_access_token);
            }
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}
