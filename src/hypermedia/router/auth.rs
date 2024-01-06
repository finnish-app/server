use crate::{
    auth::{AuthSession, LoginCredentials, SignUpCredentials},
    AppState, SignInTemplate,
};
use std::sync::Arc;

use askama_axum::IntoResponse;
use axum::{extract::State, routing::get, Json, Router};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth", get(auth_index))
        .route("/signin", get(signin_tab).post(signin))
        .route("/signup", get(signup_tab).post(signup))
}

async fn auth_index() -> impl IntoResponse {
    SignInTemplate {}
}

async fn signin_tab() -> impl IntoResponse {
    crate::hypermedia::service::auth::signin_tab().await
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
