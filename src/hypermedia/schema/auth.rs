use serde::Deserialize;

pub struct MailToUser {
    pub email: Option<String>,
    pub verification_code: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct MfaTokenForm {
    pub token: String,
}
