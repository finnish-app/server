use crate::{
    hypermedia::schema::validation::{EmailInput, PasswordInput, PasswordsInput, UsernameInput},
    AppState,
};
use std::sync::Arc;

use askama_axum::IntoResponse;
use axum::{extract::State, routing::post, Form, Router};

// VALIDATION
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/validate/email", post(validate_email))
        .route("/validate/username", post(validate_username))
        .route("/validate/passwords", post(validate_passwords))
        .route("/validate/new-passwords", post(validate_new_passwords))
        .route("/validate/password-strength", post(validate_password))
        .route(
            "/validate/new-password-strength",
            post(validate_new_password),
        )
}

async fn validate_email(
    State(shared_state): State<Arc<AppState>>,
    Form(input_email): Form<EmailInput>,
) -> impl IntoResponse {
    crate::hypermedia::service::validation::validate_email(&shared_state.pool, &input_email).await
}

async fn validate_username(Form(input_username): Form<UsernameInput>) -> impl IntoResponse {
    crate::hypermedia::service::validation::validate_username(&input_username)
}

async fn validate_passwords(Form(input_passwords): Form<PasswordsInput>) -> impl IntoResponse {
    crate::hypermedia::service::validation::validate_passwords(&input_passwords)
}

async fn validate_new_passwords(Form(input_passwords): Form<PasswordsInput>) -> impl IntoResponse {
    crate::hypermedia::service::validation::validate_new_passwords(&input_passwords)
}

async fn validate_password(Form(input_password): Form<PasswordInput>) -> impl IntoResponse {
    crate::hypermedia::service::validation::validate_password(&input_password)
}

async fn validate_new_password(Form(input_password): Form<PasswordInput>) -> impl IntoResponse {
    crate::hypermedia::service::validation::validate_new_password(&input_password)
}
