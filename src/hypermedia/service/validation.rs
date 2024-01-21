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

/// Validate that email is not taken.
/// If it is taken, returns a message to the user
/// If it is not taken, allows the user to continue
pub async fn validate_email(db_pool: &Pool<Postgres>, input: &EmailInput) -> Html<String> {
    match input.validate() {
        Ok(()) => {
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
                        return Html(format!(EMAIL_TAKEN!(), input.email));
                    }
                    return Html(format!(VALID_EMAIL!(), input.email));
                }
                Err(e) => {
                    tracing::error!("Error checking email: {}", e);
                    return Html("Error checking email".to_owned());
                }
            }
        }
        Err(_) => return Html(format!(INVALID_EMAIL!(), input.email)),
    }
}

#[derive(FromRow)]
struct Exists {
    exists: Option<bool>,
}

/// Validate that username is not taken.
/// If it is taken, returns a message to the user
/// If it is not taken, allows the user to continue
pub async fn validate_username(db_pool: &Pool<Postgres>, input: &UsernameInput) -> Html<String> {
    match input.validate() {
        Ok(()) => {
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
                        return Html(format!(USERNAME_TAKEN!(), input.username));
                    }
                    return Html(format!(VALID_USERNAME!(), input.username));
                }
                Err(e) => {
                    tracing::error!("Error checking username: {}", e);
                    return Html("Error checking username".to_owned());
                }
            }
        }
        Err(_) => return Html(format!(INVALID_USERNAME!(), input.username)),
    }
}

/// Validate that `password` and `password_confirmation` match.
/// If they don't, returns a message to the user
/// If they do, allows the user to continue
pub fn validate_passwords(input: &PasswordsInput) -> Html<String> {
    match input.validate() {
        Ok(()) => return Html(format!(MATCHING_PASSWORDS!(), input.confirm_password)),
        Err(_) => return Html(format!(MISMATCHING_PASSWORDS!(), input.confirm_password)),
    }
}

/// Validate that the new `password` and new `password_confirmation` match.
/// If they don't, returns a message to the user (the message is the same as for
/// `validate_passwords`), however, the html snippet has different input names and ids
/// If they do, allows the user to continue
pub fn validate_new_passwords(input: &PasswordsInput) -> Html<String> {
    match input.validate() {
        Ok(()) => return Html(format!(MATCHING_NEW_PASSWORDS!(), input.confirm_password)),
        Err(_) => {
            return Html(format!(
                MISMATCHING_NEW_PASSWORDS!(),
                input.confirm_password
            ))
        }
    }
}

/// Validate password strength.
/// Uses zxcvbn to check password strength
/// If it is weak, returns a message to the user
/// If it strong, allows the user to continue
pub fn validate_password(input: &PasswordInput) -> Html<String> {
    match input.validate() {
        Ok(()) => return Html(format!(STRONG_PASSWORD!(), input.password)),
        Err(_) => return Html(format!(WEAK_PASSWORD!(), input.password)),
    }
}

/// Validate new password attempt strength
/// Uses zxcvbn to check password strength
/// If it is weak, returns a message to the user (the message is the same as for
/// `validate_password`), however, the html snippet has different input names and ids
/// If it strong, allows the user to continue
pub fn validate_new_password(input: &PasswordInput) -> Html<String> {
    match input.validate() {
        Ok(()) => return Html(format!(STRONG_NEW_PASSWORD!(), input.password)),
        Err(_) => return Html(format!(WEAK_NEW_PASSWORD!(), input.password)),
    }
}
