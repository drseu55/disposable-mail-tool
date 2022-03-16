use crate::mails::MailError;
use regex::Regex;
use serde::{Deserialize, Serialize};

use reqwest;

#[derive(Serialize, Deserialize, Debug)]
pub struct GuerrillaMail {
    email_addr: String,
    email_timestamp: u64,
    alias: String,
    sid_token: String,
}

pub struct GuerrillaUser {
    phpsessid: Vec<String>,
    mails: Vec<GuerrillaMail>,
}

impl GuerrillaMail {
    pub async fn create_new_email() -> Result<Self, MailError> {
        let response = reqwest::get(
            "https://www.guerrillamail.com/ajax.php?f=get_email_address&ip=127.0.0.1&agent=Mozilla",
        )
        .await
        .unwrap();

        // Get only PHPSESSID value
        let headers = response.headers();

        let re = Regex::new(";.*$").unwrap();
        let phpsessid = re.replace(headers["set-cookie"].to_str().unwrap(), "");

        let re = Regex::new("^[A-Z]+=").unwrap();
        let phpsessid = re.replace(&phpsessid, "");

        println!("{:?}", phpsessid);

        // let mail: GuerrillaMail = response.json().await.unwrap();

        match response.status() {
            reqwest::StatusCode::OK => match response.json::<GuerrillaMail>().await {
                Ok(mail) => Ok(mail),
                Err(_) => Err(MailError::MatchError),
            },
            error => Err(MailError::ResponseError(error.to_string())),
        }
    }

    pub async fn check_email(seq: u32) {
        let response = reqwest::get(format!(
            "https://www.guerrillamail.com/ajax.php?f=check_email&seq={seq}&ip=127.0.0.1&agent=Mozilla"
        ))
        .await
        .unwrap();

        println!("{:?}", response.text().await);
    }
}
