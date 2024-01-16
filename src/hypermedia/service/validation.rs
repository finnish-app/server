use axum::response::Html;
use sqlx::{FromRow, Pool, Postgres};
use validator::Validate;

use crate::{
    constant::{
        EMAIL_TAKEN, INVALID_EMAIL, INVALID_USERNAME, MATCHING_NEW_PASSWORDS, MATCHING_PASSWORDS,
        MISMATCHING_NEW_PASSWORDS, MISMATCHING_PASSWORDS, STRONG_NEW_PASSWORD, STRONG_PASSWORD,
        USERNAME_TAKEN, VALID_EMAIL, VALID_USERNAME, WEAK_NEW_PASSWORD, WEAK_PASSWORD,
    },
    hypermedia::schema::validation::{EmailInput, PasswordInput, PasswordsInput, UsernameInput},
};

pub async fn validate_email(db_pool: &Pool<Postgres>, input: EmailInput) -> Html<String> {
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
                        Html(format!(EMAIL_TAKEN!(), input.email))
                    } else {
                        Html(format!(VALID_EMAIL!(), input.email))
                    }
                }
                Err(e) => {
                    tracing::error!("Error checking email: {}", e);
                    Html("Error checking email".to_string())
                }
            }
        }
        Err(_) => Html(format!(INVALID_EMAIL!(), input.email)),
    }
}

#[derive(FromRow)]
struct Exists {
    exists: Option<bool>,
}

pub async fn validate_username(db_pool: &Pool<Postgres>, input: UsernameInput) -> Html<String> {
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
                        Html(format!(USERNAME_TAKEN!(), input.username))
                    } else {
                        Html(format!(VALID_USERNAME!(), input.username))
                    }
                }
                Err(e) => {
                    tracing::error!("Error checking username: {}", e);
                    Html("Error checking username".to_string())
                }
            }
        }
        Err(_) => Html(format!(INVALID_USERNAME!(), input.username)),
    }
}

pub async fn validate_passwords(input: PasswordsInput) -> Html<String> {
    match input.validate() {
        Ok(_) => Html(format!(MATCHING_PASSWORDS!(), input.password)),
        Err(_) => Html(format!(MISMATCHING_PASSWORDS!(), input.password)),
    }
}

pub async fn validate_new_passwords(input: PasswordsInput) -> Html<String> {
    match input.validate() {
        Ok(_) => Html(format!(MATCHING_NEW_PASSWORDS!(), input.password)),
        Err(_) => Html(format!(MISMATCHING_NEW_PASSWORDS!(), input.password)),
    }
}

pub async fn validate_password(input: PasswordInput) -> Html<String> {
    match input.validate() {
        Ok(_) => Html(format!(STRONG_PASSWORD!(), input.password)),
        Err(_) => Html(format!(WEAK_PASSWORD!(), input.password)),
    }
}

pub async fn validate_new_password(input: PasswordInput) -> Html<String> {
    match input.validate() {
        Ok(_) => Html(format!(STRONG_NEW_PASSWORD!(), input.password)),
        Err(_) => Html(format!(WEAK_NEW_PASSWORD!(), input.password)),
    }
}
