use crate::{
    auth::{AuthSession, LoginCredentials, SignUpCredentials},
    client::mail::send_sign_up_confirmation_mail,
    constant::{SIGN_IN_TAB, SIGN_UP_TAB},
    SignInTemplate,
};

use askama_axum::IntoResponse;
use axum::{http::StatusCode, response::Html};
use password_auth::generate_hash;
use sqlx::{Pool, Postgres};

pub async fn signin(
    signin_input: LoginCredentials,
    mut auth_session: AuthSession,
) -> impl IntoResponse {
    let user = match auth_session.authenticate(signin_input).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Html("<p style=\"color:red;\">Invalid username or password</p>"),
            )
                .into_response()
        }
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    (StatusCode::OK, [("HX-Redirect", "/")], "Logged in").into_response()
}

pub async fn signin_tab(print_message: bool) -> impl IntoResponse {
    if print_message {
        tracing::info!("print was true");
        return Html(format!(
            SIGN_IN_TAB!(),
            "Account created successfully. Please sign in."
        ))
        .into_response();
    }
    tracing::info!("print was false");
    Html(format!(SIGN_IN_TAB!(), "")).into_response()
}

pub async fn signup_tab() -> impl IntoResponse {
    Html(SIGN_UP_TAB!())
}

pub async fn signup(
    db_pool: &Pool<Postgres>,
    signup_input: SignUpCredentials,
) -> impl IntoResponse {
    let hashed_pass = generate_hash(&signup_input.password);
    let mut transaction = db_pool.begin().await.unwrap();

    sqlx::query!(
        r#"INSERT INTO users (username, email, password) VALUES ($1, $2, $3)"#,
        signup_input.username,
        signup_input.email,
        hashed_pass
    )
    .execute(&mut *transaction)
    .await
    .unwrap();

    match sqlx::query!(
        "SELECT email FROM users WHERE username = $1",
        signup_input.username
    )
    .fetch_one(&mut *transaction)
    .await
    {
        Ok(email) => match send_sign_up_confirmation_mail(email.email.as_str()) {
            Ok(_) => {
                transaction.commit().await.unwrap();
                (
                    StatusCode::OK,
                    SignInTemplate {
                        should_print_signup_message_in_signin: true,
                    },
                )
                    .into_response()
            }
            Err(e) => {
                transaction.rollback().await.unwrap();
                tracing::error!("Error sending sign up confirmation mail: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
        Err(e) => {
            tracing::error!("Error signing up: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
