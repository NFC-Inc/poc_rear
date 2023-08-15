use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateWotdDto {
    pub word: String,
    pub definition: String,
    pub sentence: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DisplayWotdDto {
    pub _id: ObjectId,
    pub created_by_id: ObjectId,
    pub word: String,
    pub definition: String,
    pub sentence: String,
}
