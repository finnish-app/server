use std::sync::Arc;

use askama_axum::IntoResponse;
use axum::{extract::State, routing::get, Json, Router};

use crate::{auth::AuthSession, hypermedia::schema::auth::MfaTokenForm, AppState};

pub fn private_router() -> Router<Arc<AppState>> {
    Router::new().route("/api/auth/mfa", get(mfa_info).post(mfa_verify))
}

async fn mfa_info(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    crate::data::service::auth::mfa_info(auth_session, &shared_state.pool).await
}

async fn mfa_verify(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Json(mfa_token): Json<MfaTokenForm>,
) -> impl IntoResponse {
    crate::hypermedia::service::auth::mfa_verify(
        auth_session,
        &shared_state.pool,
        &shared_state.env,
        mfa_token.token,
    )
    .await
}
