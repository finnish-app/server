use std::sync::Arc;

use askama_axum::IntoResponse;
use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use serde::Serialize;

use crate::{auth::AuthSession, hypermedia::schema::auth::MfaTokenForm, AppState};

pub fn private_router() -> Router<Arc<AppState>> {
    Router::new().route("/api/auth/mfa", get(mfa_info).post(mfa_verify))
}

#[derive(Serialize)]
struct MfaInfo {
    otp_url: String,
}

async fn mfa_info(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
) -> Result<Json<MfaInfo>, StatusCode> {
    let Some(user) = auth_session.user else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    // TODO: create logic for changing MFA method
    if user.otp_enabled {
        todo!("Create logic for changing MFA method");
    }

    match crate::features::totp::set_otp_secret(shared_state.pool.clone(), user.id, user.email)
        .await
    {
        Ok(otp_data) => Ok(Json(MfaInfo {
            otp_url: otp_data.otp_url,
        })),
        Err(e) => {
            tracing::error!(?user.id, "Error setting OTP secret: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
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
