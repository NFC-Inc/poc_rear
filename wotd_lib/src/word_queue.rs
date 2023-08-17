use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::word_models::{DtoWotdCreate, WordModel};

/// Dto to be used to suggest a word to be added to the Queue.
#[derive(Serialize, Deserialize, Debug)]
pub struct DtoQueueItemWordSuggestNew {
    #[serde(flatten)]
    pub word: DtoWotdCreate,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DtoQueueItemWordSuggestExisting {
    pub word_id: ObjectId,
}

/// Dto to be used throughout the program when not interacting with DB.
#[derive(Serialize, Deserialize, Debug)]
pub struct DtoQueueItemWord {
    pub _id: ObjectId,
    pub word: WordModel,
    pub added_at: chrono::DateTime<chrono::Utc>,
}

/// The final product of user that will go into Database.
#[derive(Serialize, Deserialize, Debug)]
pub struct QueueItemWordModel {
    pub _id: ObjectId,
    pub word: WordModel,
    pub added_at: mongodb::bson::DateTime,
}
