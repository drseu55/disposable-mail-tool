use owo_colors::OwoColorize;
use serde_json;
use tokio;

mod mails;

const BANNER: &str = r#"
 _____  _           _____                              _       _     _     _             _             
|_   _| |__   ___  | ____|_ __   ___ _ __ ___  _   _  (_)___  | |   (_)___| |_ ___ _ __ (_)_ __   __ _ 
  | | | '_ \ / _ \ |  _| | '_ \ / _ \ '_ ` _ \| | | | | / __| | |   | / __| __/ _ \ '_ \| | '_ \ / _` |
  | | | | | |  __/ | |___| | | |  __/ | | | | | |_| | | \__ \ | |___| \__ \ ||  __/ | | | | | | | (_| |
  |_| |_| |_|\___| |_____|_| |_|\___|_| |_| |_|\__, | |_|___/ |_____|_|___/\__\___|_| |_|_|_| |_|\__, |
                                               |___/                                             |___/                                                                                                                     
"#;

fn main() -> Result<(), mails::MailError> {
    println!("{}", BANNER.fg_rgb::<0x2E, 0x31, 0x92>());
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        let (guerrilla_email, phpsessid_value) = mails::GuerrillaMail::create_new_email().await?;

        let mut guerrilla_user = mails::GuerrillaUser::new();
        guerrilla_user.email(guerrilla_email);
        guerrilla_user.phpsessid(phpsessid_value);

        let response = mails::GuerrillaMail::check_email(&guerrilla_user.phpsessid[0], 1).await?;
        println!("{}", serde_json::to_string_pretty(&response).unwrap());

        // if email.is_ok() {
        //     println!("{:?}", email);
        // } else {
        //     println!(
        //         "Something went wrong when creating email: {}",
        //         email.unwrap_err()
        //     );
        // }
        // mails::GuerrillaMail::check_email(1).await;
        Ok(())
    })
}
