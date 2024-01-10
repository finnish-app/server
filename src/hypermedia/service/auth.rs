use crate::{
    auth::{AuthSession, LoginCredentials, SignUpCredentials},
    client::mail::send_sign_up_confirmation_mail,
    constant::{SIGN_IN_TAB, SIGN_UP_TAB},
    hypermedia::schema::auth::{ChangePasswordInput, MailToUser},
    util::{generate_verification_token, now_plus_24_hours},
    SignInTemplate, VerificationTemplate,
};

use askama_axum::IntoResponse;
use axum::{
    http::StatusCode,
    response::{Html, Redirect},
};
use password_auth::generate_hash;
use sqlx::{Pool, Postgres};

pub async fn signin(
    signin_input: LoginCredentials,
    mut auth_session: AuthSession,
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

    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    (StatusCode::OK, [("HX-Redirect", "/")], "Logged in").into_response()
}

pub async fn signin_tab(print_message: bool) -> impl IntoResponse {
    if print_message {
        tracing::info!("print was true");
        return Html(format!(
            SIGN_IN_TAB!(),
            "Account created successfully. Please confirm your email and sign in."
        ))
        .into_response();
    }
    tracing::info!("print was false");
    Html(format!(SIGN_IN_TAB!(), "")).into_response()
}

pub async fn signup_tab() -> impl IntoResponse {
    Html(SIGN_UP_TAB!())
}

pub async fn signup(
    db_pool: &Pool<Postgres>,
    signup_input: SignUpCredentials,
) -> impl IntoResponse {
    let hashed_pass = generate_hash(&signup_input.password);
    let mut transaction = db_pool.begin().await.unwrap();

    sqlx::query!(
        r#"INSERT INTO users (username, email, password, verification_code, code_expires_at) VALUES ($1, $2, $3, $4, $5)"#,
        signup_input.username,
        signup_input.email,
        hashed_pass,
        generate_verification_token(),
        now_plus_24_hours()
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
            &mail_to_user.email.unwrap(),
            &mail_to_user.verification_code.unwrap(),
        ) {
            Ok(_) => {
                transaction.commit().await.unwrap();
                (
                    StatusCode::OK,
                    SignInTemplate {
                        should_print_signup_message_in_signin: true,
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
            message: "Check your inbox for your verification email".to_string(),
            login_url: "/auth".to_string(),
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
                        message: "Email verified successfully. You can now sign in.".to_string(),
                        login_url: "/auth".to_string(),
                        ..Default::default()
                    },
                )
                    .into_response(),
                Err(e) => {
                    tracing::error!("Error updating db: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        VerificationTemplate {
                            message: "Error verifying email. Please try again later.".to_string(),
                            login_url: "/auth".to_string(),
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
                    login_url: "/auth".to_string(),
                    message: "User already verified or verification code expired".to_string(),
                    should_print_resend_link: true,
                    resend_url: "/auth/resend-verification".to_string(),
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
    let user = match maybe_user {
        Some(user) => user,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let creds = LoginCredentials {
        username: user.username.clone(),
        password: change_password_input.old_password,
    };
    match auth_session.authenticate(creds).await {
        Ok(Some(user)) => {
            sqlx::query!(
                "UPDATE users SET password = $1 WHERE id = $2",
                generate_hash(&change_password_input.new_password),
                user.id
            )
            .execute(db_pool)
            .await
            .unwrap();

            StatusCode::OK.into_response()

            //Redirect::to("/auth").into_response()
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Html("<p style=\"color:red;\">Incorrect password</p>"),
            )
                .into_response()
        }
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
