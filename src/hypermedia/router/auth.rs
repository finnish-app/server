use crate::{
    auth::{AuthSession, LoginCredentials, SignUpCredentials},
    AppState, SignInTemplate,
};
use std::sync::Arc;

use askama_axum::IntoResponse;
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth", get(auth_index))
        .route("/auth/signin", get(signin_tab).post(signin))
        .route("/auth/signup", get(signup_tab).post(signup))
        .route("/auth/signin-after-signup", get(signin_tab_after_signup))
        .route("/auth/verify-email/:token", get(verify_email))
}

async fn auth_index() -> impl IntoResponse {
    SignInTemplate {
        ..Default::default()
    }
}

async fn signin_tab() -> impl IntoResponse {
    crate::hypermedia::service::auth::signin_tab(false).await
}

async fn signin_tab_after_signup() -> impl IntoResponse {
    crate::hypermedia::service::auth::signin_tab(true).await
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

async fn verify_email(
    State(shared_state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> impl IntoResponse {
    crate::hypermedia::service::auth::verify_email(&shared_state.pool, token).await
}
