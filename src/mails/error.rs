use thiserror::Error;

#[derive(Error, Debug)]
pub enum MailError {
    #[error("Query returned status code `{0}`")]
    ResponseError(String),
    #[error("Returned JSON doesn't match struct")]
    MatchError,
    #[error("Error: `{0}` when creating email")]
    CreateEmailError(String),
}

impl std::convert::From<reqwest::Error> for MailError {
    fn from(err: reqwest::Error) -> Self {
        MailError::ResponseError(err.to_string())
    }
}
