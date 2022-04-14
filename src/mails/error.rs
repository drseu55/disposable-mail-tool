use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum MailError {
    #[error("Query returned status code `{0}`")]
    ResponseError(String),
    #[error("Returned JSON doesn't match struct")]
    MatchError,
    #[error("Error: `{0}` when creating email")]
    CreateEmailError(String),
    #[error("Provider text file not found")]
    FileNotFound,
    #[error("{0}")]
    MongoDBError(String),
    #[error("{0}")]
    BsonError(String),
    #[error("{0}")]
    BsonValueAccessError(String),
    #[error("{0}")]
    BsonDeserializeError(String),
    #[error("{0}")]
    ParseIntError(String),
    #[error("{0}")]
    SerdeJsonError(String),
}

impl std::convert::From<reqwest::Error> for MailError {
    fn from(err: reqwest::Error) -> Self {
        MailError::ResponseError(err.to_string())
    }
}

impl std::convert::From<std::io::Error> for MailError {
    fn from(_err: std::io::Error) -> Self {
        MailError::FileNotFound
    }
}

impl std::convert::From<mongodb::error::Error> for MailError {
    fn from(err: mongodb::error::Error) -> Self {
        MailError::MongoDBError(err.to_string())
    }
}

impl std::convert::From<bson::ser::Error> for MailError {
    fn from(err: bson::ser::Error) -> Self {
        MailError::BsonError(err.to_string())
    }
}

impl std::convert::From<bson::document::ValueAccessError> for MailError {
    fn from(err: bson::document::ValueAccessError) -> Self {
        MailError::BsonValueAccessError(err.to_string())
    }
}

impl std::convert::From<bson::de::Error> for MailError {
    fn from(err: bson::de::Error) -> Self {
        MailError::BsonDeserializeError(err.to_string())
    }
}

impl std::convert::From<std::num::ParseIntError> for MailError {
    fn from(err: std::num::ParseIntError) -> Self {
        MailError::ParseIntError(err.to_string())
    }
}

impl std::convert::From<serde_json::Error> for MailError {
    fn from(err: serde_json::Error) -> Self {
        MailError::SerdeJsonError(err.to_string())
    }
}
