use crate::{
    auth::{AuthSession, LoginCredentials},
    client::{
        frc::{validate_frc, verify_frc_solution},
        mail::{send_forgot_password_mail, send_sign_up_confirmation_mail},
        svix::create_user_app,
    },
    features::totp::set_otp_secret,
    hypermedia::schema::{
        auth::MailToUser,
        validation::{ChangePasswordInput, Exists, ForgotPasswordInput, ResendEmail},
    },
    templates::{
        ChangePasswordTemplate, ConfirmationTemplate, ForgotPasswordTemplate, MfaTemplate,
        SignInTemplate, SignUpTemplate, VerificationTemplate,
    },
    util::{
        add_csp_to_response, generate_otp_token, generate_verification_token, now_plus_24_hours,
        now_plus_30_minutes,
    },
    Env,
};

use askama_axum::IntoResponse;
use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::{Html, Redirect},
};
use password_auth::generate_hash;
use sqlx::{Pool, Postgres};
use totp_rs::{Algorithm, Secret, TOTP};
use validator::Validate;

pub async fn signin(
    mut auth_session: AuthSession,
    env: &Env,
    signin_input: LoginCredentials,
) -> impl IntoResponse {
    if !validate_frc(&signin_input.frc_captcha_solution) {
        return (
            StatusCode::BAD_REQUEST,
            Html("<p style=\"color:red;\">Please complete the captcha</p>"),
        )
            .into_response();
    }
    if let Err(e) = verify_frc_solution(
        &signin_input.frc_captcha_solution,
        &env.frc_sitekey,
        &env.frc_apikey,
    )
    .await
    {
        tracing::error!("Error verifying frc solution: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<p style=\"color:red;\">Error verifying captcha</p>"),
        )
            .into_response();
    }

    let user = match auth_session.authenticate(signin_input).await {
        Ok(Some(user)) => {
            if !user.verified {
                return (
                    StatusCode::OK,
                    [("HX-Redirect", "/auth/email-confirmation")],
                )
                    .into_response();
            }
            user
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Html("<p style=\"color:red;\">Invalid email or password</p>"),
            )
                .into_response()
        }
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    if !user.otp_verified {
        return (StatusCode::OK, [("HX-Redirect", "/auth/mfa")]).into_response();
    }
    return (StatusCode::OK, [("HX-Redirect", "/")]).into_response();
}

pub async fn mfa_qr(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
) -> Result<Response<Body>, Response<Body>> {
    let Some(user) = auth_session.user else {
        return Err((StatusCode::UNAUTHORIZED, [("HX-Redirect", "/auth/signin")]).into_response());
    };
    tracing::debug!("User logged in");

    // TODO: create logic for changing MFA method
    if user.otp_enabled {
        todo!("Create logic for changing MFA method");
    }

    let totp = set_otp_secret(db_pool, user.id).await.map_err(|e| {
        tracing::error!(?user.id, "Error setting OTP secret: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    })?;

    Ok(MfaTemplate {
        mfa_url: "/auth/mfa".to_owned(),
        qr_code: format!("data:image/png;base64,{}", totp.qr_code),
        otp_auth_url: totp.otp_url,
        ..Default::default()
    }
    .into_response_with_nonce())
}

pub async fn mfa_verify(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    env: &Env,
    mfa_token: String,
) -> impl IntoResponse {
    let Some(user) = auth_session.user else {
        return (StatusCode::UNAUTHORIZED, [("HX-Redirect", "/auth/signin")]).into_response();
    };
    tracing::debug!("User logged in");

    let secret = Secret::Encoded(user.otp_secret.unwrap());
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.to_bytes().unwrap(),
        Some("Finnish".to_owned()),
        user.email,
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

    match create_user_app(&env.svix_api_key, user.id).await {
        Ok(app) => tracing::debug!("app: {:?}", app),
        Err(e) => {
            tracing::error!("Error creating svix app for user: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    let mut transaction = db_pool.begin().await.unwrap();
    match sqlx::query!(
        r#"
        UPDATE users SET otp_enabled = true, otp_verified = true WHERE id = $1
        "#,
        user.id
    )
    .execute(&mut *transaction)
    .await
    {
        Ok(_) => {
            sqlx::query!(
                r#"
                INSERT INTO users_groups (user_id, group_id)
                VALUES ($1, (SELECT id FROM groups WHERE name = 'user'))
                "#,
                user.id
            )
            .execute(&mut *transaction)
            .await
            .unwrap();
            transaction.commit().await.unwrap();
            return (StatusCode::OK, [("HX-Redirect", "/")]).into_response();
        }
        Err(e) => {
            transaction.rollback().await.unwrap();
            tracing::error!("Error updating db: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
}

pub fn signin_tab(env: &Env, print_message: bool) -> impl IntoResponse {
    return SignInTemplate {
        message: if print_message {
            "Password changed successfully".to_owned()
        } else {
            String::new()
        },
        frc_sitekey: env.frc_sitekey.clone(),
        ..Default::default()
    }
    .into_response_with_nonce();
}

pub fn signup_tab(env: &Env) -> impl IntoResponse {
    return SignUpTemplate {
        frc_sitekey: env.frc_sitekey.clone(),
        ..Default::default()
    }
    .into_response_with_nonce();
}

pub fn email_confirmation(env: &Env) -> impl IntoResponse {
    return ConfirmationTemplate {
        frc_sitekey: env.frc_sitekey.clone(),
        login_url: "/auth/signin".to_owned(),
        resend_url: "/auth/resend-verification".to_owned(),
        ..Default::default()
    }
    .into_response_with_nonce();
}

pub async fn resend_verification_email(
    db_pool: &Pool<Postgres>,
    env: &Env,
    resend_email: ResendEmail,
) -> impl IntoResponse {
    if !validate_frc(&resend_email.frc_captcha_solution) {
        return (
            StatusCode::BAD_REQUEST,
            Html("<p style=\"color:red;\">Please complete the captcha</p>"),
        )
            .into_response();
    }
    if let Err(e) = verify_frc_solution(
        &resend_email.frc_captcha_solution,
        &env.frc_sitekey,
        &env.frc_apikey,
    )
    .await
    {
        tracing::error!("Error verifying frc solution: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<p style=\"color:red;\">Error verifying captcha</p>"),
        )
            .into_response();
    }

    if let Err(e) = resend_email.validate() {
        tracing::error!("Error validating signup input: {}", e);
        return StatusCode::BAD_REQUEST.into_response();
    }

    if let Ok(_record) = sqlx::query_as!(
        Exists,
        r#"SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND verified = true)"#,
        resend_email.email
    )
    .fetch_one(db_pool)
    .await
    {
        return (
            StatusCode::OK,
            Html("<p style=\"color:green;\">Verification email resent successfully</p>"),
        )
            .into_response();
    }

    let mut transaction = db_pool.begin().await.unwrap();
    match sqlx::query_as!(
        MailToUser,
        r#"UPDATE users SET verification_code = $1, code_expires_at = $2 WHERE email = $3 RETURNING email, verification_code"#,
        generate_verification_token(),
        now_plus_24_hours(),
        resend_email.email
    )
    .fetch_one(&mut *transaction)
    .await
    {
        Ok(mail_to_user) => match send_sign_up_confirmation_mail(
            &crate::client::mail::EmailSecrets {
                smtp_username: &env.smtp_username,
                smtp_host: &env.smtp_host,
                smtp_key: &env.smtp_key,
                mail_from: &env.mail_from,
            },
            &mail_to_user.email.unwrap(),
            &mail_to_user.verification_code.unwrap(),
        ) {
            Ok(_) => {
                transaction.commit().await.unwrap();
                tracing::info!("Verification mail resent successfully");
                return (
                    StatusCode::OK,
                    Html("<p style=\"color:green;\">Verification email resent successfully</p>"),
                )
                    .into_response();
            }
            Err(e) => {
                transaction.rollback().await.unwrap();
                tracing::error!("Error resending verification mail: {}", e);
            }
        },
        Err(e) => {
            transaction.rollback().await.unwrap();
            tracing::error!("Error updating verification code and expiration in db: {}", e);
        }
    }

    return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Html("<p style=\"color:red;\">Error resending verification email</p>"),
    )
        .into_response();
}

pub async fn verify_email(db_pool: &Pool<Postgres>, env: &Env, token: String) -> impl IntoResponse {
    let frc_sitekey = env.frc_sitekey.clone();

    match sqlx::query!(
        "SELECT id FROM users WHERE verification_code = $1 AND code_expires_at > $2",
        token,
        chrono::Utc::now()
    )
    .fetch_one(db_pool)
    .await
    {
        Ok(returned_value) => {
            let mut transaction = db_pool.begin().await.unwrap();
            match sqlx::query!(
                "UPDATE users SET verified = true, verification_code = NULL, code_expires_at = NULL WHERE id = $1",
                returned_value.id
            )
            .execute(&mut *transaction)
            .await
            {
                Ok(_) => {
                    match sqlx::query!(
                        r#"
                        INSERT INTO users_groups (user_id, group_id)
                        VALUES ($1, (SELECT id FROM groups WHERE name = 'pseudo-user'))
                        "#,
                        returned_value.id
                    )
                    .execute(&mut *transaction)
                    .await
                    {
                        Ok(_) => {
                            transaction.commit().await.unwrap();
                            let nonce = generate_otp_token();
                            let nonce_str = format!("'nonce-{nonce}'");

                            let mut response = (
                                StatusCode::OK,
                                VerificationTemplate {
                                    frc_sitekey,
                                    login_url: "/auth/signin".to_owned(),
                                    message: "Email verified successfully. You can now sign in.".to_owned(),
                                    nonce,
                                    resend_url: "/auth/resend-verification".to_owned(),
                                    ..Default::default()
                                },
                            )
                                .into_response();

                            add_csp_to_response(&mut response, &nonce_str);
                            return response;
                        },
                        Err(e) => {
                            transaction.rollback().await.unwrap();
                            return return_error_from_email_verification(&e, frc_sitekey);
                        }
                    }
                },
                Err(e) => {
                    transaction.rollback().await.unwrap();
                    return return_error_from_email_verification(&e, frc_sitekey);
                }
            }
        }
        Err(e) => {
            tracing::error!("Error verifying email: {}", e);
            let nonce = generate_otp_token();
            let nonce_str = format!("'nonce-{nonce}'");

            let mut response = (
                StatusCode::CONFLICT,
                VerificationTemplate {
                    frc_sitekey: env.frc_sitekey.clone(),
                    login_url: "/auth/signin".to_owned(),
                    message: "User already verified or verification code expired.".to_owned(),
                    nonce,
                    resend_url: "/auth/resend-verification".to_owned(),
                    should_print_resend_link: true,
                },
            )
                .into_response();

            add_csp_to_response(&mut response, &nonce_str);
            return response;
        }
    }
}

fn return_error_from_email_verification(e: &sqlx::Error, frc_sitekey: String) -> Response<Body> {
    tracing::error!("Error updating db: {}", e);
    let nonce = generate_otp_token();
    let nonce_str = format!("'nonce-{nonce}'");

    let mut response = (
        StatusCode::INTERNAL_SERVER_ERROR,
        VerificationTemplate {
            frc_sitekey,
            login_url: "/auth/signin".to_owned(),
            message: "Error verifying email. Please try again later.".to_owned(),
            nonce,
            resend_url: "/auth/resend-verification".to_owned(),
            ..Default::default()
        },
    )
        .into_response();

    add_csp_to_response(&mut response, &nonce_str);
    return response;
}

pub async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
    match auth_session.logout().await {
        Ok(_) => Redirect::to("/auth/signin").into_response(),
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

    if let Err(e) = change_password_input.validate() {
        tracing::error!("Error validating change password input: {}", e);
        return StatusCode::BAD_REQUEST.into_response();
    }

    let creds = LoginCredentials {
        email: user.email.clone(),
        password: change_password_input.old_password,
        frc_captcha_solution: String::new(),
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
                [("HX-Redirect", "/auth/signin-after-change-password")],
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::UNAUTHORIZED,
            Html("<p style=\"color:red;\">Incorrect password</p>"),
        )
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub fn forgot_password_screen(env: &Env) -> impl IntoResponse {
    return ForgotPasswordTemplate {
        forgot_url: "/auth/forgot-password".to_owned(),
        frc_sitekey: env.frc_sitekey.clone(),
        login_url: "/auth/signin".to_owned(),
        ..Default::default()
    }
    .into_response_with_nonce();
}

pub async fn forgot_password(
    db_pool: &Pool<Postgres>,
    env: &Env,
    email_input: ResendEmail,
) -> impl IntoResponse {
    if !validate_frc(&email_input.frc_captcha_solution) {
        return (
            StatusCode::BAD_REQUEST,
            Html("<p style=\"color:red;\">Please complete the captcha</p>"),
        )
            .into_response();
    }
    if let Err(e) = verify_frc_solution(
        &email_input.frc_captcha_solution,
        &env.frc_sitekey,
        &env.frc_apikey,
    )
    .await
    {
        tracing::error!("Error verifying frc solution: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<p style=\"color:red;\">Error verifying captcha</p>"),
        )
            .into_response();
    }

    if let Err(e) = email_input.validate() {
        tracing::error!("Error validating forgot_password input: {}", e);
        return StatusCode::BAD_REQUEST.into_response();
    }

    match sqlx::query_as!(
        Exists,
        r#"SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"#,
        email_input.email
    )
    .fetch_one(db_pool)
    .await
    {
        Ok(record) => {
            if !record.exists.unwrap() {
                tracing::warn!("User with email {} does not exist", email_input.email);
                return (
                    StatusCode::OK,
                    Html("<p style=\"color:green;\">Please check your email for a link to reset your password</p>"),
                )
                    .into_response();
            }
        }
        Err(e) => {
            tracing::error!("Error checking if user with email exists: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let mut transaction = db_pool.begin().await.unwrap();
    match sqlx::query_as!(
        MailToUser,
        r#"UPDATE users SET verification_code = $1, code_expires_at = $2 WHERE email = $3 RETURNING email, verification_code"#,
        generate_verification_token(),
        now_plus_30_minutes(),
        email_input.email
    )
    .fetch_one(&mut *transaction)
    .await
    {
        Ok(mail_to_user) => match send_forgot_password_mail(
            &crate::client::mail::EmailSecrets {
                smtp_username: &env.smtp_username,
                smtp_host: &env.smtp_host,
                smtp_key: &env.smtp_key,
                mail_from: &env.mail_from,
            },
            &mail_to_user.email.unwrap(),
            &mail_to_user.verification_code.unwrap(),
        ) {
            Ok(_) => {
                transaction.commit().await.unwrap();
                tracing::info!("Forgot password mail sent successfully");
            }
            Err(e) => {
                transaction.rollback().await.unwrap();
                tracing::error!("Error resending forgot password mail: {}", e);
            }
        },
        Err(e) => {
            transaction.rollback().await.unwrap();
            tracing::error!("Error updating verification code and expiration in db: {}", e);
        }
    }

    return (
        StatusCode::OK,
        Html("<p style=\"color:green;\">Please check your email for a link to reset your password</p>"),
    )
        .into_response();
}

pub fn change_forgotten_password_screen(secret_code: &str) -> impl IntoResponse {
    return ChangePasswordTemplate {
        change_password: format!("/auth/reset-password/{secret_code}"),
        passwords_match: "/validate/new-passwords".to_owned(),
        password_strength: "/validate/new-password-strength".to_owned(),
        forgot_password: true,
        ..Default::default()
    }
    .into_response_with_nonce();
}

pub async fn change_forgotten_password(
    db_pool: &Pool<Postgres>,
    forgot_password_input: ForgotPasswordInput,
    secret_code: String,
) -> impl IntoResponse {
    match sqlx::query!(
        "SELECT id FROM users WHERE verification_code = $1 AND code_expires_at > $2",
        secret_code,
        chrono::Utc::now()
    )
    .fetch_one(db_pool)
    .await
    {
        Ok(returned_value) => {
            match sqlx::query!(
                "UPDATE users SET password = $1, verification_code = NULL, code_expires_at = NULL WHERE id = $2",
                generate_hash(&forgot_password_input.password),
                returned_value.id
            )
            .execute(db_pool)
            .await
            {
                Ok(_) => {
                    tracing::info!("Password changed successfully");
                    return (
                        StatusCode::OK,
                        [("HX-Redirect", "/auth/signin-after-change-password")],
                    )
                        .into_response();
                }
                Err(e) => {
                    tracing::error!("Error updating db: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Html("<p style=\"color:red;\">Error changing forgotten password</p>"),
                    ).into_response();
                }
            }
        }
        Err(e) => {
            tracing::error!("Error changing forgotten password: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("<p style=\"color:red;\">Error changing forgotten password</p>"),
            ).into_response();
        }
    }
}
