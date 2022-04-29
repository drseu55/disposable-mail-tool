use bson;
use mongodb::{options::ClientOptions, options::IndexOptions, Client, Collection, IndexModel};
use std::time;

use crate::mails;

pub async fn connect(url: &str, port: &str) -> Result<Client, mongodb::error::Error> {
    let mut client_options = ClientOptions::parse(format!("{url}:{port}")).await?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_db_connection() -> Result<(), mongodb::error::Error> {
        let client = connect("mongodb://localhost", "27017").await;
        assert!(client.is_ok());
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_db_connection_with_wrong_data() -> Result<(), mongodb::error::Error> {
        let client = connect("mongodb://localhost", "some_port").await;
        assert!(client.is_err());
        Ok(())
    }
}
