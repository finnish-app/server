use crate::{
    auth::{AuthSession, LoginCredentials, SignUpCredentials},
    constant::{SIGN_IN_TAB, SIGN_UP_TAB},
};

use askama_axum::IntoResponse;
use axum::{http::StatusCode, response::Html, Json};
use password_auth::generate_hash;
use sqlx::{Pool, Postgres};

pub async fn signin(
    Json(signin_input): Json<LoginCredentials>,
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

pub async fn signin_tab() -> impl IntoResponse {
    Html(SIGN_IN_TAB!())
}

pub async fn signup_tab() -> impl IntoResponse {
    Html(SIGN_UP_TAB!())
}

pub async fn signup(
    db_pool: &Pool<Postgres>,
    Json(signup_input): Json<SignUpCredentials>,
) -> impl IntoResponse {
    let hashed_pass = generate_hash(&signup_input.password);
    match sqlx::query!(
        r#"INSERT INTO users (username, email, password) VALUES ($1, $2, $3) RETURNING id"#,
        signup_input.username,
        signup_input.email,
        hashed_pass
    )
    .fetch_one(db_pool)
    .await
    {
        Ok(_) => (StatusCode::OK, [("HX-Redirect", "/auth")], "Created user").into_response(),
        Err(e) => {
            tracing::error!("Error signing up: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
