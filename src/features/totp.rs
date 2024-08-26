use crate::util::generate_otp_token;
use anyhow::bail;
use sqlx::{Pool, Postgres};
use totp_rs::{Algorithm, Secret, TOTP};

pub struct OtpData {
    /// `qr_code` is a base64 encoded image that can be rendered embedded in an <img> tag
    pub qr_code: String,
    /// `otp_url` is a otpauth:// url that can be rendered as a QR code
    pub otp_url: String,
}

pub async fn set_otp_secret(db_pool: &Pool<Postgres>, user_id: i32) -> anyhow::Result<OtpData> {
    let secret = Secret::Raw(generate_otp_token().as_bytes().to_vec());
    let mut transaction = db_pool.begin().await?;

    let user_email = sqlx::query!(
        r#"
        UPDATE users SET otp_secret = $1 WHERE id = $2
        RETURNING email
        "#,
        secret.to_encoded().to_string(),
        user_id
    )
    .fetch_one(&mut *transaction)
    .await?;

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.to_bytes().unwrap(),
        Some("Finnish".to_owned()),
        user_email.email,
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
