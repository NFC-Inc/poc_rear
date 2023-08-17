use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// What is required when creating a new user.
#[derive(Serialize, Deserialize)]
pub struct DtoUserCreate {
    pub username: String,
    pub password: String,
    pub email: String,
}

/// What is required when a user is logging in.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DtoUserLogin {
    pub username: String,
    pub password: String,
}

/// Dto to be used throughout the program when not interacting with DB.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DtoUser {
    pub _id: ObjectId,
    pub username: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// The final product of user that will go into Database.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserModel {
    pub _id: ObjectId,
    pub username: String,
    pub password: String,
    pub email: String,
    pub created_at: mongodb::bson::DateTime,
    pub updated_at: mongodb::bson::DateTime,
}

impl From<UserModel> for DtoUser {
    fn from(user_model: UserModel) -> Self {
        DtoUser {
            _id: user_model._id,
            username: user_model.username,
            email: user_model.email,
            created_at: user_model.created_at.into(),
            updated_at: user_model.updated_at.into(),
        }
    }
}

#[cfg(test)]
mod model_tests {
    #[test]
    fn mongo_to_chrono_datetime() {
        let mongo_dt = mongodb::bson::DateTime::now();
        let chrono_dt = chrono::DateTime::from(mongo_dt);
        assert_eq!(chrono_dt.timestamp_millis(), mongo_dt.timestamp_millis());
    }

    #[test]
    fn chrono_to_mongo_datetime() {
        let chrono_dt = chrono::Utc::now();
        let mongo_dt = mongodb::bson::DateTime::from(chrono_dt);
        assert_eq!(chrono_dt.timestamp_millis(), mongo_dt.timestamp_millis());
    }
}
