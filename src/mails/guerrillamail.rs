use crate::mails::MailError;
use mongodb::bson::oid;
use regex::Regex;
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
    pub phpsessid: Vec<String>,
    pub mails: Vec<GuerrillaMail>,
}

impl GuerrillaMail {
    pub async fn create_new_email() -> Result<(Self, String), MailError> {
        let response = match reqwest::get(
            "https://www.guerrillamail.com/ajax.php?f=get_email_address&ip=127.0.0.1&agent=Mozilla",
        )
        .await
        {
            Ok(response) => response,
            Err(e) => return Err(MailError::CreateEmailError(e.to_string())),
        };

        // Get only PHPSESSID value
        let headers = response.headers().clone();

        let re = Regex::new(";.*$").unwrap();
        let phpsessid = re.replace(headers["set-cookie"].to_str().unwrap(), "");

        let re = Regex::new("^[A-Z]+=").unwrap();
        let phpsessid = re.replace(&phpsessid, "");

        match response.status() {
            reqwest::StatusCode::OK => match response.json::<GuerrillaMail>().await {
                Ok(mail) => Ok((mail, phpsessid.to_string())),
                Err(_) => Err(MailError::MatchError),
            },
            error => Err(MailError::ResponseError(error.to_string())),
        }
    }

    pub async fn check_email(
        phpsessid_value: &String,
        seq: u32,
        sid_token: &String,
    ) -> Result<String, reqwest::Error> {
        let client = Client::builder().build()?;
        let response = client
            .get(format!(
            "https://www.guerrillamail.com/ajax.php?f=check_email&seq={seq}&sid_token={sid_token}"
        ))
            .header("Cookie", format!("PHPSESSID={phpsessid_value}"))
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
            phpsessid: Vec::new(),
            mails: Vec::new(),
        }
    }

    pub fn email(&mut self, email: GuerrillaMail) {
        self.mails.push(email);
    }

    pub fn phpsessid(&mut self, phpsessid_value: String) {
        self.phpsessid.push(phpsessid_value);
    }
}
