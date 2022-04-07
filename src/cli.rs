use clap::{arg, Command};

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
            Command::new("fetch")
                .about("Fetches emails from inbox")
                .arg(arg!(-e --email <EMAIL> "Email address"))
                .arg(arg!(-c --count <COUNT> "Count of emails to fetch - max is 20"))
                .arg_required_else_help(true),
        )
}
