use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use validator::{Validate, ValidationError};
use zxcvbn::{zxcvbn, Score};

static RE_USERNAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-z0-9]{3,20}$").unwrap());

#[derive(Deserialize, Validate, Debug)]
pub struct EmailInput {
    #[validate(email)]
    pub email: String,
}

#[derive(Deserialize, Validate)]
pub struct UsernameInput {
    #[validate(regex(path = *RE_USERNAME))]
    pub username: String,
}

#[derive(Deserialize, Validate)]
pub struct PasswordInput {
    #[validate(custom(function = "validate_password_strength"))]
    pub password: String,
}

fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    match zxcvbn(password, &[]).score() {
        Score::Zero | Score::One | Score::Two => Err(ValidationError::new("Password is too weak")),
        Score::Three | Score::Four => Ok(()),
        _ => unreachable!(),
    }
}

#[derive(Deserialize, Validate)]
pub struct PasswordsInput {
    pub password: String,
    #[validate(must_match(other = "password"))]
    pub confirm_password: String,
}

#[derive(Deserialize, Validate)]
pub struct SignUpInput {
    #[validate(regex(path = *RE_USERNAME))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(custom(function = "validate_password_strength"))]
    pub password: String,
    #[validate(must_match(other = "password"))]
    pub confirm_password: String,
    #[serde(rename = "frc-captcha-solution")]
    pub frc_captcha_solution: String,
}

#[derive(Deserialize, Validate)]
pub struct ChangePasswordInput {
    pub old_password: String,
    #[validate(custom(function = "validate_password_strength"))]
    pub password: String,
    #[validate(must_match(other = "password"))]
    pub confirm_password: String,
}

#[derive(Deserialize, Validate)]
pub struct ForgotPasswordInput {
    #[validate(custom(function = "validate_password_strength"))]
    pub password: String,
    #[validate(must_match(other = "password"))]
    pub confirm_password: String,
}

#[derive(sqlx::FromRow)]
pub struct Exists {
    pub exists: Option<bool>,
}

#[derive(Deserialize, Validate)]
pub struct ResendEmail {
    #[validate(email)]
    pub email: String,
    #[serde(rename = "frc-captcha-solution")]
    pub frc_captcha_solution: String,
}
