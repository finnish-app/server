use crate::{
    auth::{AuthSession, LoginCredentials, SignUpCredentials},
    hypermedia::schema::auth::ChangePasswordInput,
    AppState, AuthTemplate, ChangePasswordTemplate,
};
use std::sync::Arc;

use askama_axum::IntoResponse;
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};

pub fn public_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth", get(auth_index))
        .route("/auth/signin", get(signin_tab).post(signin))
        .route("/auth/signup", get(signup_tab).post(signup))
        .route("/auth/signin-after-signup", get(signin_tab_after_signup))
        .route(
            "/auth/signin-after-change-password",
            get(signin_tab_after_change_password),
        )
        .route("/auth/verify-email/:token", get(verify_email))
        .route("/auth/resend-verification", get(resend_verification_email))
        .route("/auth/forgot-password", get(forgot_password))
}

pub fn private_router() -> Router<Arc<AppState>> {
    Router::new().route("/auth/logout", get(logout)).route(
        "/auth/change-password",
        get(change_password_screen).post(change_password),
    )
}

async fn auth_index() -> impl IntoResponse {
    AuthTemplate {
        ..Default::default()
    }
}

async fn signin_tab() -> impl IntoResponse {
    crate::hypermedia::service::auth::signin_tab(0).await
}

async fn signin_tab_after_signup() -> impl IntoResponse {
    crate::hypermedia::service::auth::signin_tab(1).await
}

async fn signin_tab_after_change_password() -> impl IntoResponse {
    crate::hypermedia::service::auth::signin_tab(2).await
}

async fn signin(
    auth_session: AuthSession,
    Json(signin_input): Json<LoginCredentials>,
) -> impl IntoResponse {
    crate::hypermedia::service::auth::signin(signin_input, auth_session).await
}

async fn signup_tab() -> impl IntoResponse {
    crate::hypermedia::service::auth::signup_tab().await
}

async fn signup(
    State(shared_state): State<Arc<AppState>>,
    Json(signup_input): Json<SignUpCredentials>,
) -> impl IntoResponse {
    crate::hypermedia::service::auth::signup(&shared_state.pool, signup_input).await
}

#[derive(serde::Deserialize, Debug)]
pub struct ResendEmailUsername {
    username: String,
}

async fn resend_verification_email(
    State(shared_state): State<Arc<AppState>>,
    Query(resend_email_username): Query<ResendEmailUsername>,
) -> impl IntoResponse {
    crate::hypermedia::service::auth::resend_verification_email(
        &shared_state.pool,
        resend_email_username.username,
    )
    .await
}

async fn verify_email(
    State(shared_state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> impl IntoResponse {
    crate::hypermedia::service::auth::verify_email(&shared_state.pool, token).await
}

async fn forgot_password() -> impl IntoResponse {
    //crate::hypermedia::service::auth::forgot_password().await
    todo!()
}

async fn logout(auth_session: AuthSession) -> impl IntoResponse {
    crate::hypermedia::service::auth::logout(auth_session).await
}

async fn change_password_screen() -> impl IntoResponse {
    ChangePasswordTemplate {
        url: "/auth/change-password".to_string(),
    }
}

async fn change_password(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Json(change_password_input): Json<ChangePasswordInput>,
) -> impl IntoResponse {
    crate::hypermedia::service::auth::change_password(
        auth_session,
        &shared_state.pool,
        change_password_input,
    )
    .await
}
