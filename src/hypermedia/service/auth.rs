use crate::{
    auth::{AuthSession, LoginCredentials, SignUpCredentials, User},
    client::mail::send_sign_up_confirmation_mail,
    constant::{SIGN_IN_TAB, SIGN_UP_TAB},
    hypermedia::schema::auth::{ChangePasswordInput, MailToUser},
    templates::{AuthTemplate, MfaTemplate, VerificationTemplate},
    util::{generate_otp_token, generate_verification_token, now_plus_24_hours},
};

use askama_axum::IntoResponse;
use axum::{
    http::StatusCode,
    response::{Html, Redirect},
};
use password_auth::generate_hash;
use shuttle_secrets::SecretStore;
use sqlx::{Pool, Postgres};
use totp_rs::{Algorithm, Secret, TOTP};

async fn generate_otp(db_pool: &Pool<Postgres>, user: &User) -> impl IntoResponse {
    let secret = Secret::Raw(generate_otp_token().as_bytes().to_vec());
    tracing::info!("Secret: {:?}", secret);

    let mut transaction = db_pool.begin().await.unwrap();

    sqlx::query!(
        r#"
        UPDATE users SET otp_secret = $1 WHERE id = $2
        "#,
        secret.to_string(),
        user.id
    )
    .execute(&mut *transaction)
    .await
    .unwrap();

    let totp = TOTP::new(
        Algorithm::SHA256,
        6,
        1,
        30,
        secret.to_bytes().unwrap(),
        Some("Finnish".to_owned()),
        user.email.clone(),
    )
    .unwrap();
    let qr_code = totp.get_qr_base64().unwrap(); // qr_code is a base64 encoded image
                                                 // that can be rendered embedded in an <img> tag

    transaction.commit().await.unwrap();
    return MfaTemplate {
        mfa_url: format!("/auth/mfa?username={}", user.username),
        qr_code: format!("data:image/png;base64,{qr_code}"),
    };
}

pub async fn signin(
    mut auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    signin_input: LoginCredentials,
) -> impl IntoResponse {
    let user = match auth_session.authenticate(signin_input).await {
        Ok(Some(user)) => {
            if !user.verified {
                return (
                    StatusCode::UNAUTHORIZED,
                    Html(format!(
                        "<p style=\"color:red;\">Please verify your email before signing in</p>
                         <a href=\"/auth/resend-verification?username={}\">Resend verification email</a>
                        ",
                        user.username
                    )),
                )
                    .into_response();
            }
            user
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Html("<p style=\"color:red;\">Invalid username or password</p>"),
            )
                .into_response()
        }
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if !user.otp_enabled {
        let qr = generate_otp(db_pool, &user).await;
        return qr.into_response();
    }

    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    (StatusCode::OK, [("HX-Redirect", "/")]).into_response()
}

pub async fn mfa_verify(
    db_pool: &Pool<Postgres>,
    username: String,
    mfa_token: String,
) -> impl IntoResponse {
    let record = sqlx::query!(
        r#"
        SELECT email, otp_secret
        FROM users
        WHERE username = $1
        "#,
        username
    )
    .fetch_one(db_pool)
    .await
    .unwrap();

    let totp = TOTP::new(
        Algorithm::SHA256,
        6,
        1,
        30,
        record.otp_secret.unwrap().as_bytes().to_vec(),
        Some("Finnish".to_owned()),
        record.email.clone(),
    )
    .unwrap();

    match totp.check_current(&mfa_token) {
        Ok(boolean) => {
            if !boolean {
                return (
                    StatusCode::UNAUTHORIZED,
                    Html("<p style=\"color:red;\">Invalid MFA token</p>"),
                )
                    .into_response();
            }
        }
        Err(e) => {
            tracing::error!("Error verifying MFA token: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    sqlx::query!(
        r#"
        UPDATE users SET otp_enabled = true, otp_verified = true WHERE username = $1
        "#,
        username
    )
    .execute(db_pool)
    .await
    .unwrap();

    (StatusCode::OK, [("HX-Redirect", "/auth")]).into_response()
}

pub fn signin_tab(print_message: u8) -> Html<String> {
    match print_message.cmp(&1) {
        std::cmp::Ordering::Greater => {
            Html(format!(SIGN_IN_TAB!(), "Password changed successfully."))
        }
        std::cmp::Ordering::Equal => Html(format!(
            SIGN_IN_TAB!(),
            "Account created successfully. Please confirm your email and sign in."
        )),
        std::cmp::Ordering::Less => Html(format!(SIGN_IN_TAB!(), "")),
    }
}

pub const fn signup_tab() -> Html<&'static str> {
    Html(SIGN_UP_TAB!())
}

pub async fn signup(
    db_pool: &Pool<Postgres>,
    secret_store: &SecretStore,
    signup_input: SignUpCredentials,
) -> impl IntoResponse {
    let hashed_pass = generate_hash(&signup_input.password);
    let mut transaction = db_pool.begin().await.unwrap();

    let Some(expiration_date) = now_plus_24_hours() else {
        transaction.rollback().await.unwrap();
        tracing::error!("Error generating expiration date, maybe it overflowed?");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    sqlx::query!(
        r#"INSERT INTO users (username, email, password, verification_code, code_expires_at) VALUES ($1, $2, $3, $4, $5)"#,
        signup_input.username,
        signup_input.email,
        hashed_pass,
        generate_verification_token(),
        expiration_date
    )
    .execute(&mut *transaction)
    .await
    .unwrap();

    match sqlx::query_as!(
        MailToUser,
        "SELECT email, verification_code FROM users WHERE username = $1",
        signup_input.username
    )
    .fetch_one(&mut *transaction)
    .await
    {
        Ok(mail_to_user) => match send_sign_up_confirmation_mail(
            secret_store,
            &mail_to_user.email.unwrap(),
            &mail_to_user.verification_code.unwrap(),
        ) {
            Ok(_) => {
                transaction.commit().await.unwrap();
                (
                    StatusCode::OK,
                    AuthTemplate {
                        should_print_message_in_signin: 1,
                    },
                )
                    .into_response()
            }
            Err(e) => {
                transaction.rollback().await.unwrap();
                tracing::error!("Error sending sign up confirmation mail: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
        Err(e) => {
            tracing::error!("Error signing up: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn resend_verification_email(
    db_pool: &Pool<Postgres>,
    secret_store: &SecretStore,
    username: String,
) -> impl IntoResponse {
    match sqlx::query_as!(
        MailToUser,
        r#"UPDATE users SET verification_code = $1, code_expires_at = $2 WHERE username = $3 RETURNING email, verification_code"#,
        generate_verification_token(),
        now_plus_24_hours(),
        username
    )
    .fetch_one(db_pool)
    .await
    {
        Ok(mail_to_user) => match send_sign_up_confirmation_mail(
            secret_store,
            &mail_to_user.email.unwrap(),
            &mail_to_user.verification_code.unwrap(),
        ) {
            Ok(_) => tracing::info!("Verification mail resent successfully"),
            Err(e) => {
                tracing::error!("Error resending verification mail: {}", e);
            }
        },
        Err(e) => {
            tracing::error!("Error updating verification code and expiration in db: {}", e);
        }
    }

    (
        StatusCode::OK,
        VerificationTemplate {
            message: "Check your inbox for your verification email".to_owned(),
            login_url: "/auth".to_owned(),
            ..Default::default()
        },
    )
        .into_response()
}

pub async fn verify_email(db_pool: &Pool<Postgres>, token: String) -> impl IntoResponse {
    match sqlx::query!(
        "SELECT id FROM users WHERE verification_code = $1 AND code_expires_at > $2",
        token,
        chrono::Utc::now()
    )
    .fetch_one(db_pool)
    .await
    {
        Ok(returned_value) => {
            match sqlx::query!(
                "UPDATE users SET verified = true, verification_code = NULL, code_expires_at = NULL WHERE id = $1",
                returned_value.id
            )
            .execute(db_pool)
            .await
            {
                Ok(_) => (
                    StatusCode::OK,
                    VerificationTemplate {
                        message: "Email verified successfully. You can now sign in.".to_owned(),
                        login_url: "/auth".to_owned(),
                        ..Default::default()
                    },
                )
                    .into_response(),
                Err(e) => {
                    tracing::error!("Error updating db: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        VerificationTemplate {
                            message: "Error verifying email. Please try again later.".to_owned(),
                            login_url: "/auth".to_owned(),
                            ..Default::default()
                        },
                    ).into_response()
                }
            }
        }
        Err(e) => {
            tracing::error!("Error verifying email: {}", e);
            (
                StatusCode::CONFLICT,
                VerificationTemplate {
                    login_url: "/auth".to_owned(),
                    message: "User already verified or verification code expired".to_owned(),
                    should_print_resend_link: true,
                    resend_url: "/auth/resend-verification".to_owned(),
                },
            ).into_response()
        }
    }
}

pub async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
    match auth_session.logout().await {
        Ok(_) => Redirect::to("/auth").into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn change_password(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    change_password_input: ChangePasswordInput,
) -> impl IntoResponse {
    let maybe_user = &auth_session.user;
    let Some(user) = maybe_user else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let creds = LoginCredentials {
        username: user.username.clone(),
        password: change_password_input.old_password,
    };
    match auth_session.authenticate(creds).await {
        Ok(Some(user)) => {
            sqlx::query!(
                "UPDATE users SET password = $1 WHERE id = $2",
                generate_hash(&change_password_input.password),
                user.id
            )
            .execute(db_pool)
            .await
            .unwrap();

            (
                StatusCode::OK,
                [("HX-Push-Url", "/auth")],
                AuthTemplate {
                    should_print_message_in_signin: 2,
                },
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Html("<p style=\"color:red;\">Incorrect password</p>"),
        )
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
