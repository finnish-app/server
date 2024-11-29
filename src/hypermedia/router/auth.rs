use crate::{
    auth::{AuthSession, LoginCredentials},
    hypermedia::schema::{
        auth::MfaTokenForm,
        validation::{ChangePasswordInput, ForgotPasswordInput, ResendEmail, SignUpInput},
    },
    templates::ChangePasswordTemplate,
    AppState,
};
use std::sync::Arc;

use askama_axum::IntoResponse;
use axum::{
    extract::{Path, State},
    response::{Html, Response},
    routing::{get, post},
    Form, Router,
};
use reqwest::StatusCode;

pub fn public_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/signin", get(signin_tab).post(signin))
        .route("/auth/signup", get(signup_tab).post(signup))
        .route(
            "/auth/signin-after-change-password",
            get(signin_tab_after_change_password),
        )
        .route("/auth/email-confirmation", get(email_confirmation))
        .route("/auth/verify-email/:token", get(verify_email))
        .route("/auth/resend-verification", post(resend_verification_email))
        .route(
            "/auth/forgot-password",
            get(forgot_password_screen).post(forgot_password),
        )
        .route(
            "/auth/reset-password/:token",
            get(change_forgotten_password_screen).post(change_forgotten_password),
        )
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

async fn signin_tab(State(shared_state): State<Arc<AppState>>) -> impl IntoResponse {
    crate::hypermedia::service::auth::signin_tab(&shared_state.env, false)
}

async fn signin_tab_after_change_password(
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    crate::hypermedia::service::auth::signin_tab(&shared_state.env, true)
}

async fn signin(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Form(signin_input): Form<LoginCredentials>,
) -> impl IntoResponse {
    crate::hypermedia::service::auth::signin(auth_session, &shared_state.env, signin_input).await
}

async fn email_confirmation(State(shared_state): State<Arc<AppState>>) -> impl IntoResponse {
    crate::hypermedia::service::auth::email_confirmation(&shared_state.env)
}

async fn mfa_qr(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    crate::hypermedia::service::auth::mfa_qr(auth_session, shared_state.pool.clone()).await
}

async fn mfa_verify(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Form(mfa_token): Form<MfaTokenForm>,
) -> impl IntoResponse {
    crate::hypermedia::service::auth::mfa_verify(
        auth_session,
        &shared_state.pool,
        &shared_state.env,
        mfa_token.token,
    )
    .await
}

async fn signup_tab(State(shared_state): State<Arc<AppState>>) -> impl IntoResponse {
    crate::hypermedia::service::auth::signup_tab(&shared_state.env)
}

async fn signup(
    State(shared_state): State<Arc<AppState>>,
    Form(signup_input): Form<SignUpInput>,
) -> Result<impl IntoResponse, Response> {
    let outcome =
        crate::features::user::create(shared_state.pool.clone(), &shared_state.env, signup_input)
            .await
            .map_err(|e| {
                tracing::error!("Error inserting expense: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            })?;

    match outcome {
        crate::features::user::CreateOutcome::PendingCaptcha => Err((
            StatusCode::BAD_REQUEST,
            Html("<p style=\"color:red;\">Please complete the captcha</p>"),
        )
            .into_response()),
        crate::features::user::CreateOutcome::InvalidCaptcha(e) => {
            tracing::error!("Error verifying frc solution: {}", e);
            Err((
                StatusCode::FAILED_DEPENDENCY,
                Html(format!(
                    "<p style=\"color:red;\">Error verifying captcha {e}</p>"
                )),
            )
                .into_response())
        }
        crate::features::user::CreateOutcome::InvalidInput(e) => Err((
            StatusCode::PRECONDITION_FAILED,
            Html(format!(
                "<p style=\"color:red;\">Please send valid inputs {e}</p>"
            )),
        )
            .into_response()),
        crate::features::user::CreateOutcome::Success
        | crate::features::user::CreateOutcome::EmailConfirmationResent
        | crate::features::user::CreateOutcome::EmailAlreadyConfirmed => {
            Ok([("HX-Redirect", "/auth/email-confirmation")])
        }
    }
}

async fn resend_verification_email(
    State(shared_state): State<Arc<AppState>>,
    Form(resend_email): Form<ResendEmail>,
) -> impl IntoResponse {
    crate::hypermedia::service::auth::resend_verification_email(
        &shared_state.pool,
        &shared_state.env,
        resend_email,
    )
    .await
}

async fn verify_email(
    State(shared_state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> impl IntoResponse {
    crate::hypermedia::service::auth::verify_email(&shared_state.pool, &shared_state.env, token)
        .await
}

async fn forgot_password_screen(State(shared_state): State<Arc<AppState>>) -> impl IntoResponse {
    crate::hypermedia::service::auth::forgot_password_screen(&shared_state.env)
}

async fn forgot_password(
    State(shared_state): State<Arc<AppState>>,
    Form(email): Form<ResendEmail>,
) -> impl IntoResponse {
    crate::hypermedia::service::auth::forgot_password(&shared_state.pool, &shared_state.env, email)
        .await
}

async fn logout(auth_session: AuthSession) -> impl IntoResponse {
    crate::hypermedia::service::auth::logout(auth_session).await
}

async fn change_password_screen() -> impl IntoResponse {
    return ChangePasswordTemplate {
        change_password: "/auth/change-password".to_owned(),
        passwords_match: "/validate/new-passwords".to_owned(),
        password_strength: "/validate/new-password-strength".to_owned(),
        forgot_password: false,
        ..Default::default()
    }
    .into_response_with_nonce();
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

async fn change_forgotten_password_screen(Path(token): Path<String>) -> impl IntoResponse {
    crate::hypermedia::service::auth::change_forgotten_password_screen(&token)
}

async fn change_forgotten_password(
    State(shared_state): State<Arc<AppState>>,
    Path(token): Path<String>,
    Form(change_password_input): Form<ForgotPasswordInput>,
) -> impl IntoResponse {
    crate::hypermedia::service::auth::change_forgotten_password(
        &shared_state.pool,
        change_password_input,
        token,
    )
    .await
}
