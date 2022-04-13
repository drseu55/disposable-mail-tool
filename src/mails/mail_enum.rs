use super::GuerrillaUser;

#[derive(Clone)]
pub enum MailEnum {
    Guerrilla(GuerrillaUser),
    NotAvailabe(String),
}
