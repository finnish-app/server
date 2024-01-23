use crate::{
    auth::{AuthSession, LoginCredentials, SignUpCredentials},
    hypermedia::schema::auth::{ChangePasswordInput, MfaTokenForm},
    templates::{AuthTemplate, ChangePasswordTemplate},
    AppState,
};
use std::sync::Arc;

use askama_axum::IntoResponse;
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Form, Router,
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
    Router::new()
        .route("/auth/logout", get(logout))
        .route(
            "/auth/change-password",
            get(change_password_screen).post(change_password),
        )
        .route("/auth/mfa", get(mfa_qr).post(mfa_verify))
}

async fn auth_index() -> impl IntoResponse {
    AuthTemplate::default()
}

async fn signin_tab() -> impl IntoResponse {
    crate::hypermedia::service::auth::signin_tab(0)
}

async fn signin_tab_after_signup() -> impl IntoResponse {
    crate::hypermedia::service::auth::signin_tab(1)
}

async fn signin_tab_after_change_password() -> impl IntoResponse {
    crate::hypermedia::service::auth::signin_tab(2)
}

async fn signin(
    auth_session: AuthSession,
    Form(signin_input): Form<LoginCredentials>,
) -> impl IntoResponse {
    crate::hypermedia::service::auth::signin(auth_session, signin_input).await
}

async fn mfa_qr(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    crate::hypermedia::service::auth::mfa_qr(auth_session, &shared_state.pool).await
}

async fn mfa_verify(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Form(mfa_token): Form<MfaTokenForm>,
) -> impl IntoResponse {
    crate::hypermedia::service::auth::mfa_verify(auth_session, &shared_state.pool, mfa_token.token)
        .await
}

async fn signup_tab() -> impl IntoResponse {
    crate::hypermedia::service::auth::signup_tab()
}

async fn signup(
    State(shared_state): State<Arc<AppState>>,
    Form(signup_input): Form<SignUpCredentials>,
) -> impl IntoResponse {
    crate::hypermedia::service::auth::signup(
        &shared_state.pool,
        &shared_state.secret_store,
        signup_input,
    )
    .await
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
        &shared_state.secret_store,
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
        change_password: "/auth/change-password".to_owned(),
        passwords_match: "/validate/new-passwords".to_owned(),
        password_strength: "/validate/new-password-strength".to_owned(),
    }
}

async fn change_password(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Form(change_password_input): Form<ChangePasswordInput>,
) -> impl IntoResponse {
    crate::hypermedia::service::auth::change_password(
        auth_session,
        &shared_state.pool,
        change_password_input,
    )
    .await
}
