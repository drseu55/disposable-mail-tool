use mongodb::{options::ClientOptions, Client};

const URL: &str = "mongodb://localhost";
const PORT: &str = "27017";

pub async fn connect() -> Result<Client, mongodb::error::Error> {
    let mut client_options = ClientOptions::parse(format!("{URL}:{PORT}")).await?;

    client_options.app_name = Some("disposable_email".to_string());

    let client = Client::with_options(client_options)?;

    Ok(client)
}
