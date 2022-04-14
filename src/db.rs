use bson;
use mongodb::{options::ClientOptions, options::IndexOptions, Client, Collection, IndexModel};
use std::time;

use crate::mails;

const URL: &str = "mongodb://localhost";
const PORT: &str = "27017";

pub async fn connect() -> Result<Client, mongodb::error::Error> {
    let mut client_options = ClientOptions::parse(format!("{URL}:{PORT}")).await?;

    client_options.app_name = Some("disposable_email".to_string());

    let client = Client::with_options(client_options)?;

    Ok(client)
}

pub async fn create_index(
    email_users: &Collection<bson::Document>,
) -> Result<(), mails::MailError> {
    // Create an index in collection that will automatically
    // delete document after 60 minutes
    let index_key = bson::doc! { "createdAt": 1 };
    let index_options = IndexOptions::builder()
        .expire_after(Some(time::Duration::new(3600, 0)))
        .build();
    let index_model = IndexModel::builder()
        .keys(index_key)
        .options(index_options)
        .build();

    email_users.create_index(index_model, None).await?;

    Ok(())
}
