use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateUser {
    pub username: String,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub _id: ObjectId,
    pub username: String,
}
