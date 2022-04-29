use bson;
use chrono::prelude::*;
use clap::{arg, Command};
use comfy_table::Table;
use mongodb;
use owo_colors::colors::*;
use owo_colors::OwoColorize;
use serde_json;
use tokio;

use std::fs;
use std::time::Duration;

use crate::db;
use crate::mails;
use crate::mails::GuerrillaUser;

const FILENAME: &str = "providers.txt";
const URL: &str = "mongodb://localhost";
const PORT: &str = "27017";

pub fn cli() -> Command<'static> {
    Command::new("disposable_mail")
        .about("Tool for generating disposable emails from different email providers")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("list").about("List available email providers"))
        .subcommand(Command::new("guerrillamails").about("List unexpired guerillamails from database"))
        .subcommand(
            Command::new("create")
                .about("Creates new email address")
                .arg(arg!(<PROVIDER> "Email provider"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("get")
                .about("Fetches available emails")
                .arg(arg!(-'e' --"email" <EMAIL> "Email address"))
                .arg_required_else_help(true)
                .arg(arg!(-'o' --"offset" <OFFSET> "How many emails to start from. Ex: Offset of 0 will fetch a list of the first 10 emails"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("check")
                .about("Checks for new email")
                .arg(arg!(-'e' --"email" <EMAIL> "Email address"))
                .arg_required_else_help(true)
                .arg(arg!(-'c' --"count" <COUNT> "The sequence number (id) of the oldest email"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("fetch")
                .about("Fetches email information")
                .arg(arg!(-'e' --"email" <EMAIL> "Email address"))
                .arg_required_else_help(true)
                .arg(arg!(--"id" <ID> "Id of the received email from inbox"))
                .arg_required_else_help(true)
        )
}

pub async fn menu() -> Result<(), mails::MailError> {
    let args = cli().get_matches();

    let mongodb_client = db::connect(URL, PORT).await?;
    let db = mongodb_client.database("disposable_mail_db");

    match args.subcommand() {
        Some(("list", _)) => {
            println!("{}", list_providers(FILENAME)?);
        }
        Some(("guerrillamails", _)) => {
            let emails = mails::get_unexpired_guerrillamails_from_db(&db).await?;

            if emails.is_empty() {
                println!("There is not available guerrillamails");
            }

            for email_addr in emails {
                println!("{}", email_addr);
            }
        }
        Some(("create", sub_args)) => {
            let provider = sub_args.value_of("PROVIDER").expect("required");

            let provider_struct = create_email_from_provider(provider).await?;

            let email_users = db.collection::<bson::Document>("email_users");

            db::create_index(&email_users).await?;

            match provider_struct {
                mails::MailEnum::Guerrilla(guerrilla_user) => {
                    let serialized_guerrilla_mail = bson::to_bson(&guerrilla_user)?;

                    // It is safe to call unwrap on this result because
                    // serializing a struct to BSON (above function) creates a BSON document type.
                    let document = serialized_guerrilla_mail.as_document().unwrap();

                    email_users.insert_one(document.to_owned(), None).await?;

                    println!(
                        "Your guerrilla temp email: {}",
                        guerrilla_user.mails[0].email_addr
                    );
                    println!("{}", "Emails expire after 60 minutes".fg::<BrightYellow>());
                }
                _ => panic!("Unexpected struct"),
            }
        }
        Some(("get", sub_args)) => {
            let email = sub_args.value_of("email").expect("required");
            let seq = sub_args.value_of("offset").expect("required");

            let seq: u32 = seq.parse()?;

            let response = check_available_emails_from_provider(&db, "get", email, seq).await?;

            pretty_print_json(response);
        }
        Some(("check", sub_args)) => {
            let email = sub_args.value_of("email").expect("required");
            let seq = sub_args.value_of("count").expect("required");

            let seq: u32 = seq.parse()?;

            let response = check_available_emails_from_provider(&db, "check", email, seq).await?;

            pretty_print_json(response);
        }
        Some(("fetch", sub_args)) => {
            let email = sub_args.value_of("email").expect("required");
            let email_id = sub_args.value_of("id").expect("required");

            let response = fetch_email_from_provider(&db, email, email_id).await?;

            print_fetched_email(response);
        }
        _ => println!("No such argument"),
    }

    Ok(())
}

fn list_providers(filename: &str) -> Result<String, mails::MailError> {
    let providers = fs::read_to_string(filename)?;
    Ok(providers)
}

async fn create_email_from_provider(provider: &str) -> Result<mails::MailEnum, mails::MailError> {
    match provider {
        "guerrillamail" => {
            let guerrilla_email = mails::GuerrillaMail::create_new_email().await?;

            // Using unwrap is safe here, because unix timestamp
            // does not gonna exceed i64 soon
            let mail_creation_date = chrono::DateTime::from_utc(
                NaiveDateTime::from_timestamp(
                    guerrilla_email.email_timestamp.try_into().unwrap(),
                    100_000_000,
                ),
                Utc,
            );

            let mut guerrilla_user = mails::GuerrillaUser::new(mail_creation_date);

            guerrilla_user.email(guerrilla_email);

            Ok(mails::MailEnum::Guerrilla(guerrilla_user))
        }
        _ => Ok(mails::MailEnum::NotAvailabe(
            "Email provider not available".to_string(),
        )),
    }
}

/// Searches database to find the correct values
/// and deserialize them to correct struct
async fn check_available_emails_from_provider(
    db: &mongodb::Database,
    call_function: &str,
    email: &str,
    seq: u32,
) -> Result<Vec<serde_json::Value>, mails::MailError> {
    // Check if email address is in database
    // If it is not, it means that user did not run create first
    let found_obj = find_element_in_db(db, "email_users", "mails.email_addr", email).await?;

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

                    let result_response = if call_function == "get" {
                        let response = mails::GuerrillaMail::get_email_list(
                            seq,
                            &guerrilla_user_struct.mails[index].sid_token,
                        )
                        .await?;

                        let value: serde_json::Value = serde_json::from_str(&response)?;

                        // Using unwrap is safe here if Guerrillamail API does not change return type
                        // Currently returns a vector
                        let list = value["list"].as_array().unwrap();

                        list.to_vec()
                    } else {
                        // Check every 10 seconds if returned list
                        // from response has data
                        // Break after 5 minutes (30 ticks) if list is still empty
                        println!(
                            "Breaks automatically after 5 minutes if there is not a new email"
                        );

                        let mut i = tokio::time::interval(Duration::from_secs(10));
                        let mut counter = 0;

                        loop {
                            i.tick().await;

                            counter += 1;

                            if counter == 30 {
                                break vec![];
                            }

                            let response = mails::GuerrillaMail::check_email(
                                seq,
                                &guerrilla_user_struct.mails[index].sid_token,
                            )
                            .await?;

                            let value: serde_json::Value = serde_json::from_str(&response)?;

                            // Using unwrap is safe here if Guerrillamail API does not change return type
                            // Currently returns a vector
                            let list = value["list"].as_array().unwrap();

                            if list.is_empty() {
                                println!("Checking for new email...");
                            } else {
                                break list.to_vec();
                            }
                        }
                    };

                    Ok(result_response)
                }
                _ => panic!("Unexpected email provider"),
            }
        }
        None => Err(mails::MailError::EmailCheckError),
    }
}

async fn fetch_email_from_provider(
    db: &mongodb::Database,
    email: &str,
    email_id: &str,
) -> Result<serde_json::Value, mails::MailError> {
    let found_obj = find_element_in_db(db, "email_users", "mails.email_addr", email).await?;

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

                    let response = mails::GuerrillaMail::fetch_email(
                        email_id,
                        &guerrilla_user_struct.mails[index].sid_token,
                    )
                    .await?;

                    let value: serde_json::Value = serde_json::from_str(&response)?;

                    Ok(value)
                }
                _ => panic!("Unexpected email provider"),
            }
        }
        None => Err(mails::MailError::EmailCheckError),
    }
}

async fn find_element_in_db(
    db: &mongodb::Database,
    collection: &str,
    key: &str,
    value: &str,
) -> Result<Option<bson::Document>, mails::MailError> {
    let email_users = db.collection::<bson::Document>(collection);

    let filter = bson::doc! {key: value};

    let found_obj = email_users.find_one(filter, None).await?;

    Ok(found_obj)
}

fn pretty_print_json(json_data: Vec<serde_json::Value>) {
    let mut table = Table::new();

    table.set_header(vec!["ID", "From", "Subject", "Date"]);

    if json_data.is_empty() {
        println!("");
        return;
    }

    for value in json_data {
        // Using unwrap is safe here, because mail_timestamp always contains digits
        // and does not gonna exceed u64 soon
        let timestamp = value["mail_timestamp"]
            .as_str()
            .unwrap()
            .parse::<i64>()
            .unwrap();

        let date: DateTime<Utc> =
            chrono::DateTime::from_utc(NaiveDateTime::from_timestamp(timestamp, 0), Utc);

        table.add_row(vec![
            &value["mail_id"].to_string(),
            &value["mail_from"].to_string(),
            &value["mail_subject"].to_string(),
            &date.to_string(),
        ]);
    }

    println!("{table}");
}

fn print_fetched_email(value: serde_json::Value) {
    if value == false {
        println!("Unexpected email id");
        return;
    }

    // Using unwrap is safe here
    // because check for empty email is above
    println!("From: {}", value["mail_from"].as_str().unwrap());
    println!("Date: {} UTC", value["mail_date"].as_str().unwrap());
    println!("Subject: {}", value["mail_subject"].as_str().unwrap());
    println!("\n{}", value["mail_body"].as_str().unwrap());
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
    #[ignore]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_find_element_in_db() -> Result<(), mails::MailError> {
        let mongodb_client = db::connect(URL, PORT).await?;
        let db = mongodb_client.database("disposable_mail_db");

        let found =
            find_element_in_db(&db, "email_users", "mails.email_addr", "some_value").await?;

        assert_eq!(found, None);

        Ok(())
    }
}
