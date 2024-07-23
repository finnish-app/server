use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::{auth::AuthSession, features::totp::set_otp_secret};

#[derive(Serialize, Deserialize)]
pub struct MfaInfo {
    pub otp_url: String,
}

pub async fn mfa_info(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
) -> Result<Json<MfaInfo>, StatusCode> {
    let Some(user) = auth_session.user else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    tracing::info!("User logged in");

    // TODO: create logic for changing MFA method
    if user.otp_enabled {
        todo!("Create logic for changing MFA method");
    }

    let totp = set_otp_secret(db_pool, user.id).await.map_err(|e| {
        tracing::error!(?user.id, "Error setting OTP secret: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR;
    })?;

    Ok(Json(MfaInfo {
        otp_url: totp.otp_url,
    }))
}
