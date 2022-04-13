use bson;
use clap::{arg, Command};
use futures::stream::TryStreamExt;
use mongodb::{options, Client};
use owo_colors::colors::*;
use owo_colors::OwoColorize;

use std::fs;

use crate::db;
use crate::mails;
use crate::mails::GuerrillaUser;

const FILENAME: &str = "providers.txt";

pub fn cli() -> Command<'static> {
    Command::new("disposable_mail")
        .about("Tool for generating disposable emails from different email providers")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("list").about("List available email providers"))
        .subcommand(
            Command::new("create")
                .about("Creates new email address")
                .arg(arg!(<PROVIDER> "Email provider"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("send")
                .about("Sends new email")
                .arg(arg!(-r --receiver <RECEIVER> "Type email to receive your message"))
                .arg(arg!(-m --message <MESSAGE> "File which content will be send"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("check")
                .about("Checks available emails from inbox")
                .arg(arg!(-'e' --"email" <EMAIL> "Email address"))
                .arg_required_else_help(true)
                .arg(arg!(-'c' --"count" <COUNT> "The sequence number (id) of the oldest email - max 20"))
                .arg_required_else_help(true),
        )
}

pub async fn menu() -> Result<(), mails::MailError> {
    let args = cli().get_matches();

    let mongodb_client = db::connect().await?;
    let db = mongodb_client.database("disposable_mail_db");

    match args.subcommand() {
        Some(("list", _)) => {
            println!("{}", list_providers(FILENAME)?);
        }
        Some(("create", sub_args)) => {
            let provider = sub_args.value_of("PROVIDER").expect("required");

            let provider_struct = create_email_from_provider(provider).await?;

            match provider_struct {
                mails::MailEnum::Guerrilla(guerrilla_user) => {
                    let email_users = db.collection::<bson::Document>("emails_user");

                    if db
                        .list_collection_names(None)
                        .await?
                        .contains(&"emails_user".to_string())
                    {
                        // If there is a guerrillamail collection, update mails list
                        let serialized_guerrilla_mail = bson::to_bson(&guerrilla_user.mails[0])?;

                        // It is safe to call unwrap on this result because
                        // serializing a struct to BSON (above function) creates a BSON document type.
                        let document_mail = serialized_guerrilla_mail.as_document().unwrap();

                        let query = bson::doc! { "name": "guerrillamail" };
                        let update = bson::doc! { "$push": { "mails": document_mail } };

                        // TODO: Consider another approach to eliminate variable cloning
                        email_users.update_one(query.clone(), update, None).await?;

                        let update =
                            bson::doc! { "$push": { "phpsessid": &guerrilla_user.phpsessid[0] } };

                        email_users.update_one(query, update, None).await?;
                    } else {
                        // If there is not a guerrillamail collection, create new collection and add data
                        let serialized_guerrilla_user = bson::to_bson(&guerrilla_user)?;

                        // It is safe to call unwrap on this result because
                        // serializing a struct to BSON (above function) creates a BSON document type.
                        let document = serialized_guerrilla_user.as_document().unwrap();

                        email_users.insert_one(document.to_owned(), None).await?;
                    }

                    // TODO: Add some type of loading to tell user that email is generating
                    println!(
                        "Your guerrilla temp email: {}",
                        guerrilla_user.mails[0].email_addr
                    );
                    println!("{}", "Emails expire after 60 minutes".fg::<BrightYellow>());
                }
                _ => panic!("Unexpected struct"),
            }
        }
        Some(("check", sub_args)) => {
            let email = sub_args.value_of("email").expect("required");
            let seq = sub_args.value_of("count").expect("required");

            // TODO: Handle parse error properly
            let seq: u32 = seq.parse().unwrap();

            let response = check_available_emails_from_provider(&db, email, seq).await?;

            // TODO: Handle serde_json error properly
            println!("{}", serde_json::to_string_pretty(&response).unwrap());
        }
        _ => println!("No such argument"),
    }

    Ok(())
}

fn list_providers(filename: &str) -> Result<String, mails::MailError> {
    // TODO: Add description for each email provider in text file
    let providers = fs::read_to_string(filename)?;
    Ok(providers)
}

async fn create_email_from_provider(provider: &str) -> Result<mails::MailEnum, mails::MailError> {
    match provider {
        "guerrillamail" => {
            let (guerrilla_email, phpsessid_value) =
                mails::GuerrillaMail::create_new_email().await?;

            let mut guerrilla_user = mails::GuerrillaUser::new();

            guerrilla_user.email(guerrilla_email);
            guerrilla_user.phpsessid(phpsessid_value);

            Ok(mails::MailEnum::Guerrilla(guerrilla_user))
        }
        _ => Ok(mails::MailEnum::NotAvailabe(
            "Email provider not available".to_string(),
        )),
    }
}

// Should search db to find the correct values
// and make a correct object
async fn check_available_emails_from_provider(
    db: &mongodb::Database,
    email: &str,
    seq: u32,
) -> Result<String, mails::MailError> {
    // Check if email address is in database
    // If it is not, it means that user did not run create first
    // TODO: Check if email expired (current timestamp and timestamp from db)
    // If it is expired, call set_email_address endpoint to set same address
    let emails_user = db.collection::<bson::Document>("emails_user");

    let filter = bson::doc! {"mails.email_addr": email};

    let found_obj = emails_user.find_one(filter, None).await?;

    match found_obj {
        Some(email_obj) => {
            let name_str = email_obj.get_str("name")?;

            match name_str {
                "guerrillamail" => {
                    let guerrilla_user_struct: GuerrillaUser =
                        bson::from_bson(bson::Bson::Document(email_obj))?;

                    let index = guerrilla_user_struct
                        .mails
                        .iter()
                        .position(|x| x.email_addr == email)
                        .unwrap();

                    let response = mails::GuerrillaMail::check_email(
                        &guerrilla_user_struct.phpsessid[index],
                        seq,
                        &guerrilla_user_struct.mails[index].sid_token,
                    )
                    .await?;

                    Ok(response)
                }
                _ => panic!("Unexpected email provider"),
            }
        }
        None => Ok("Email address is not in database.
            You are passing a wrong email address or you did not call create first"
            .to_string()),
    }
}

#[cfg(test)]
mod tests {
    use crate::mails::MailError;

    use super::*;

    #[test]
    fn test_list_providers_success() {
        assert!(list_providers(FILENAME).is_ok());
    }

    #[test]
    fn test_list_providers_err() {
        match list_providers("somefile") {
            Err(e) => assert_eq!(e, MailError::FileNotFound),
            _ => panic!("Unexpected error"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_guerrillamail_creation() -> Result<(), mails::MailError> {
        let email = create_email_from_provider("guerrillamail").await;
        assert!(!email.is_err());
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_not_found_provider_email_creation() -> Result<(), mails::MailError> {
        let email = create_email_from_provider("example").await?;
        match email {
            mails::MailEnum::NotAvailabe(msg) => {
                assert_eq!(msg, "Email provider not available".to_string())
            }
            _ => panic!("Unexpected message"),
        }
        Ok(())
    }
}
