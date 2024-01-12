use askama_axum::IntoResponse;
use axum::response::Html;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use sqlx::{FromRow, Pool, Postgres};
use validator::{Validate, ValidationError};
use zxcvbn::zxcvbn;

use crate::constant::{
    EMAIL_TAKEN, INVALID_EMAIL, INVALID_USERNAME, MATCHING_NEW_PASSWORDS, MATCHING_PASSWORDS,
    MISMATCHING_NEW_PASSWORDS, MISMATCHING_PASSWORDS, STRONG_NEW_PASSWORD, STRONG_PASSWORD,
    USERNAME_TAKEN, VALID_EMAIL, VALID_USERNAME, WEAK_NEW_PASSWORD, WEAK_PASSWORD,
};

lazy_static! {
    static ref RE_USERNAME: Regex = Regex::new(r"^[a-z0-9]{3,20}$").unwrap();
}

#[derive(Deserialize, Validate, Debug)]
pub struct EmailInput {
    #[validate(email)]
    email: String,
}

#[derive(Deserialize, Validate)]
pub struct UsernameInput {
    #[validate(regex = "RE_USERNAME")]
    username: String,
}

#[derive(Deserialize, Validate)]
pub struct PasswordInput {
    #[validate(custom = "validate_password_strength")]
    password: String,
}

fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    if zxcvbn(password, &[]).unwrap().score() < 3 {
        return Err(ValidationError::new("Password is too weak"));
    }
    Ok(())
}

#[derive(Deserialize, Validate)]
pub struct PasswordsInput {
    password: String,
    #[validate(must_match = "password")]
    confirm_password: String,
}

pub async fn validate_email(db_pool: &Pool<Postgres>, input: EmailInput) -> impl IntoResponse {
    match input.validate() {
        Ok(_) => {
            match sqlx::query_as!(
                Exists,
                r#"SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"#,
                input.email
            )
            .fetch_one(db_pool)
            .await
            {
                Ok(exists) => {
                    if exists.exists.unwrap() {
                        Html(format!(EMAIL_TAKEN!(), input.email)).into_response()
                    } else {
                        Html(format!(VALID_EMAIL!(), input.email)).into_response()
                    }
                }
                Err(e) => {
                    tracing::error!("Error checking email: {}", e);
                    Html("Error checking email").into_response()
                }
            }
        }
        Err(_) => Html(format!(INVALID_EMAIL!(), input.email)).into_response(),
    }
}

#[derive(FromRow)]
struct Exists {
    exists: Option<bool>,
}

pub async fn validate_username(
    db_pool: &Pool<Postgres>,
    input: UsernameInput,
) -> impl IntoResponse {
    match input.validate() {
        Ok(_) => {
            match sqlx::query_as!(
                Exists,
                r#"SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)"#,
                input.username
            )
            .fetch_one(db_pool)
            .await
            {
                Ok(exists) => {
                    if exists.exists.unwrap() {
                        Html(format!(USERNAME_TAKEN!(), input.username)).into_response()
                    } else {
                        Html(format!(VALID_USERNAME!(), input.username)).into_response()
                    }
                }
                Err(e) => {
                    tracing::error!("Error checking username: {}", e);
                    Html("Error checking username").into_response()
                }
            }
        }
        Err(_) => Html(format!(INVALID_USERNAME!(), input.username)).into_response(),
    }
}

pub async fn validate_passwords(input: PasswordsInput) -> impl IntoResponse {
    match input.validate() {
        Ok(_) => Html(format!(MATCHING_PASSWORDS!(), input.password)).into_response(),
        Err(_) => Html(format!(MISMATCHING_PASSWORDS!(), input.password)).into_response(),
    }
}

pub async fn validate_new_passwords(input: PasswordsInput) -> impl IntoResponse {
    match input.validate() {
        Ok(_) => Html(format!(MATCHING_NEW_PASSWORDS!(), input.password)).into_response(),
        Err(_) => Html(format!(MISMATCHING_NEW_PASSWORDS!(), input.password)).into_response(),
    }
}

pub async fn validate_password(input: PasswordInput) -> impl IntoResponse {
    match input.validate() {
        Ok(_) => Html(format!(STRONG_PASSWORD!(), input.password)).into_response(),
        Err(_) => Html(format!(WEAK_PASSWORD!(), input.password)).into_response(),
    }
}

pub async fn validate_new_password(input: PasswordInput) -> impl IntoResponse {
    match input.validate() {
        Ok(_) => Html(format!(STRONG_NEW_PASSWORD!(), input.password)).into_response(),
        Err(_) => Html(format!(WEAK_NEW_PASSWORD!(), input.password)).into_response(),
    }
}
