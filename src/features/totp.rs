use crate::util::generate_otp_token;
use anyhow::bail;
use sqlx::PgPool;
use totp_rs::{Algorithm, Secret, TOTP};

pub struct OtpData {
    /// `qr_code` is a base64 encoded image that can be rendered embedded in an <img> tag
    pub qr_code: String,
    /// `otp_url` is a otpauth:// url that can be rendered as a QR code
    pub otp_url: String,
}

pub async fn set_otp_secret(
    db_pool: PgPool,
    user_id: i32,
    user_email: String,
) -> anyhow::Result<OtpData> {
    let secret = Secret::Raw(generate_otp_token().as_bytes().to_vec());
    let mut transaction = db_pool.begin().await?;

    crate::queries::user::set_otp_secret(&mut *transaction, user_id, &secret)
        .await
        .map(|c| {
            if c.rows_affected() > 1 {
                tracing::error!("i really need a macro that cancels the transaction");
            }
        })?;

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.to_bytes().unwrap(),
        Some("Finnish".to_owned()),
        user_email,
    )?;

    transaction.commit().await?;

    let Ok(qr_code) = totp.get_qr_base64() else {
        bail!("Failed to generate QR code from totp");
    };

    Ok(OtpData {
        qr_code,
        otp_url: totp.get_url(),
    })
}
