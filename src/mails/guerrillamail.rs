use crate::mails::MailError;
use regex::Regex;
use serde::{Deserialize, Serialize};

use reqwest::Client;

#[derive(Serialize, Deserialize, Debug)]
pub struct GuerrillaMail {
    email_addr: String,
    email_timestamp: u64,
    alias: String,
    sid_token: String,
}

pub struct GuerrillaUser {
    pub phpsessid: Vec<String>,
    mails: Vec<GuerrillaMail>,
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

    // async fn set_email_address(email_user: String, lang: String) {

    // }

    pub async fn check_email(phpsessid_value: &String, seq: u32) -> Result<String, reqwest::Error> {
        let client = Client::builder().build()?;
        let response = client.get(format!(
            "https://www.guerrillamail.com/ajax.php?f=check_email&seq={seq}&ip=127.0.0.1&agent=Mozilla"
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
