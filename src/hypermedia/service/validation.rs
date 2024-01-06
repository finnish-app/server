use askama_axum::IntoResponse;
use axum::response::Html;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use sqlx::{FromRow, Pool, Postgres};
use validator::Validate;

use crate::constant::{
    EMAIL_TAKEN, INVALID_EMAIL, INVALID_USERNAME, MATCHING_PASSWORDS, MISMATCHING_PASSWORDS,
    USERNAME_TAKEN, VALID_EMAIL, VALID_USERNAME,
};

lazy_static! {
    static ref RE_USERNAME: Regex = Regex::new(r"^[a-z0-9]{3,20}$").unwrap();
}

#[derive(Deserialize, Validate)]
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
