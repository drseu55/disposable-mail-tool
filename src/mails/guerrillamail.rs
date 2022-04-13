use crate::mails::MailError;
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
}

impl GuerrillaUser {
    pub fn new() -> Self {
        GuerrillaUser {
            id: None,
            name: "guerrillamail".to_string(),
            mails: Vec::new(),
        }
    }

    pub fn email(&mut self, email: GuerrillaMail) {
        self.mails.push(email);
    }
}
