use thiserror::Error;

#[derive(Error, Debug)]
pub enum MailError {
    #[error("Query returned status code `{0}`")]
    ResponseError(String),
    #[error("Returned JSON doesn't match struct")]
    MatchError,
}

impl std::convert::From<reqwest::Error> for MailError {
    fn from(err: reqwest::Error) -> Self {
        MailError::ResponseError(err.to_string())
    }
}
