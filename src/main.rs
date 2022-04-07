use owo_colors::OwoColorize;
use serde_json;
use tokio;

use std::fs;

mod cli;
mod mails;

const FILENAME: &str = "providers.txt";

const BANNER: &str = r#"
 _____  _           _____                              _       _     _     _             _             
|_   _| |__   ___  | ____|_ __   ___ _ __ ___  _   _  (_)___  | |   (_)___| |_ ___ _ __ (_)_ __   __ _ 
  | | | '_ \ / _ \ |  _| | '_ \ / _ \ '_ ` _ \| | | | | / __| | |   | / __| __/ _ \ '_ \| | '_ \ / _` |
  | | | | | |  __/ | |___| | | |  __/ | | | | | |_| | | \__ \ | |___| \__ \ ||  __/ | | | | | | | (_| |
  |_| |_| |_|\___| |_____|_| |_|\___|_| |_| |_|\__, | |_|___/ |_____|_|___/\__\___|_| |_|_|_| |_|\__, |
                                               |___/                                             |___/                                                                                                                     
"#;

#[tokio::main]
async fn main() -> Result<(), mails::MailError> {
    println!("{}", BANNER.fg_rgb::<0x2E, 0x31, 0x92>());

    let args = cli::cli().get_matches();

    match args.subcommand() {
        Some(("list", _)) => {
            list_providers()?;
        }
        Some(("create", sub_args)) => {
            let provider = sub_args.value_of("PROVIDER").expect("required");
            create_email_from_provider(provider).await?
        }
        _ => println!("No such argument"),
    }

    Ok(())
    //     // if email.is_ok() {
    //     //     println!("{:?}", email);
    //     // } else {
    //     //     println!(
    //     //         "Something went wrong when creating email: {}",
    //     //         email.unwrap_err()
    //     //     );
    //     // }
    //     // mails::GuerrillaMail::check_email(1).await;
}

fn list_providers() -> Result<(), mails::MailError> {
    // TODO: Add description for each email provider in text file
    let providers = fs::read_to_string(FILENAME)?;

    println!("{}", providers);

    Ok(())
}

async fn create_email_from_provider(provider: &str) -> Result<(), mails::MailError> {
    match provider {
        "guerrillamail" => {
            let (guerrilla_email, phpsessid_value) =
                mails::GuerrillaMail::create_new_email().await?;

            let mut guerrilla_user = mails::GuerrillaUser::new();

            guerrilla_user.email(guerrilla_email);
            guerrilla_user.phpsessid(phpsessid_value);

            println!(
                "Your guerrilla temp email: {}",
                guerrilla_user.mails[0].email_addr
            );

            Ok(())
        }
        _ => {
            println!("Email provider not available");
            Ok(())
        }
    }
}

fn fetch_inbox_from_provider() {
    // let response =
    //     mails::GuerrillaMail::check_email(&guerrilla_user.phpsessid[0], 1).await?;
    // println!("{}", serde_json::to_string_pretty(&response).unwrap());
    unimplemented!()
}
