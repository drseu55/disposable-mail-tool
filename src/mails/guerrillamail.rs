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
    pub async fn create_new_email() -> Self {
        let response = reqwest::get(
            "https://www.guerrillamail.com/ajax.php?f=get_email_address&ip=127.0.0.1&agent=Mozilla",
        )
        .await
        .unwrap();

        // println!("{:?}", mail);

        let mail: GuerrillaMail = response.json().await.unwrap();

        mail
    }
}
