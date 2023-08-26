use axum::{extract::Path, http::StatusCode, response::Response, Extension, Form};
use config_lib::config::Config;
use mongodb::Client;
use user_lib::{
    user_logic::{create_new_user, get_one_user},
    user_models::{DtoUserCreate, UserModel},
};

pub async fn create_user(
    Extension(client): Extension<std::sync::Arc<Client>>,
    Form(create_user_form): Form<DtoUserCreate>,
) -> Result<Response, StatusCode> {
    let collection = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_USERS);

    create_new_user(collection, create_user_form).await
}

pub async fn get_user(
    Extension(client): Extension<std::sync::Arc<Client>>,
    Path(username): Path<String>,
) -> Result<Response, StatusCode> {
    let collection: mongodb::Collection<UserModel> = client
        .database(Config::MONGO_DB_NAME)
        .collection(Config::MONGO_COLL_NAME_USERS);

    get_one_user(collection, username).await
}
