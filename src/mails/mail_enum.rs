use super::GuerrillaUser;

#[derive(Clone, Debug, PartialEq)]
pub enum MailEnum {
    Guerrilla(GuerrillaUser),
    NotAvailabe(String),
}
