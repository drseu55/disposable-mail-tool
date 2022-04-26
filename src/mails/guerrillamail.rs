use crate::mails::MailError;
use bson;
use chrono::prelude::*;
use futures::stream::TryStreamExt;
use mongodb::bson::oid;
use serde::{Deserialize, Serialize};

use reqwest::Client;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuerrillaMail {
    pub email_addr: String,
    pub email_timestamp: u64,
    pub alias: String,
    pub sid_token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuerrillaUser {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<oid::ObjectId>,
    #[serde(
        rename = "createdAt",
        default = "date_default_value",
        skip_deserializing,
        with = "bson::serde_helpers::chrono_datetime_as_bson_datetime"
    )]
    pub created_at: chrono::DateTime<Utc>,
    pub name: String,
    pub mails: Vec<GuerrillaMail>,
}

impl GuerrillaMail {
    pub async fn create_new_email() -> Result<Self, MailError> {
        let response = match reqwest::get(
            "https://www.guerrillamail.com/ajax.php?f=get_email_address&ip=127.0.0.1&agent=Mozilla",
        )
        .await
        {
            Ok(response) => response,
            Err(e) => return Err(MailError::CreateEmailError(e.to_string())),
        };

        match response.status() {
            reqwest::StatusCode::OK => match response.json::<GuerrillaMail>().await {
                Ok(mail) => Ok(mail),
                Err(_) => Err(MailError::MatchError),
            },
            error => Err(MailError::ResponseError(error.to_string())),
        }
    }

    pub async fn check_email(seq: u32, sid_token: &String) -> Result<String, reqwest::Error> {
        let client = Client::builder().build()?;
        let response = client
            .get(format!(
            "https://www.guerrillamail.com/ajax.php?f=check_email&seq={seq}&sid_token={sid_token}"
        ))
            .header("Cookie", format!("PHPSESSID={sid_token}"))
            .send()
            .await?;

        Ok(response.text().await?)
    }

    pub async fn get_email_list(seq: u32, sid_token: &String) -> Result<String, reqwest::Error> {
        let client = Client::builder().build()?;
        let response = client
            .get(format!(
            "https://www.guerrillamail.com/ajax.php?f=get_email_list&offset={seq}&sid_token={sid_token}&seq=1"
        ))
            .header("Cookie", format!("PHPSESSID={sid_token}"))
            .send()
            .await?;

        Ok(response.text().await?)
    }

    pub async fn fetch_email(email_id: &str, sid_token: &String) -> Result<String, reqwest::Error> {
        let client = Client::builder().build()?;
        let response = client
            .get(format!(
            "https://www.guerrillamail.com/ajax.php?f=fetch_email&email_id={email_id}&sid_token={sid_token}"
        ))
            .header("Cookie", format!("PHPSESSID={sid_token}"))
            .send()
            .await?;

        Ok(response.text().await?)
    }
}

impl GuerrillaUser {
    pub fn new(mail_creation_date: chrono::DateTime<Utc>) -> Self {
        GuerrillaUser {
            id: None,
            created_at: mail_creation_date,
            name: "guerrillamail".to_string(),
            mails: Vec::new(),
        }
    }

    pub fn email(&mut self, email: GuerrillaMail) {
        self.mails.push(email);
    }
}

fn date_default_value() -> chrono::DateTime<Utc> {
    chrono::Utc::now()
}

pub async fn get_unexpired_guerrillamails_from_db(
    db: &mongodb::Database,
) -> Result<Vec<String>, MailError> {
    let email_users = db.collection::<bson::Document>("email_users");

    let options = mongodb::options::FindOptions::builder()
        .projection(bson::doc! { "mails.email_addr": 1, "_id": 0 })
        .build();

    let mut found_emails = email_users.find(bson::doc! {}, options).await?;

    let mut emails: Vec<bson::Document> = Vec::new();

    while let Some(email) = found_emails.try_next().await? {
        emails.push(email);
    }

    if emails.is_empty() {
        return Ok(Vec::new());
    }

    let mut email_addrs: Vec<String> = Vec::new();

    for email in emails {
        // Using unwrap is safe, because mongodb fields are known
        if email.get("mails").is_some() {
            let emails_bson = email.get("mails").unwrap().as_array().unwrap();
            for email in emails_bson {
                let email_addr = email
                    .as_document()
                    .unwrap()
                    .get("email_addr")
                    .unwrap()
                    .as_str()
                    .unwrap();

                email_addrs.push(email_addr.to_owned());
            }
        } else {
            return Ok(Vec::new());
        };
    }

    Ok(email_addrs)
}
