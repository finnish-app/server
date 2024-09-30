use anyhow::bail;
use password_auth::generate_hash;
use sqlx::PgPool;
use validator::{Validate, ValidationErrors};

use crate::{
    client::{
        frc::{validate_frc, verify_frc_solution, FrcVerificationErrorList},
        mail::{send_forgot_password_mail, send_sign_up_confirmation_mail, EmailSecrets},
    },
    hypermedia::schema::validation::SignUpInput,
    util::{generate_verification_token, now_plus_24_hours},
    Env,
};

pub enum CreateOutcome {
    PendingCaptcha,
    InvalidCaptcha(FrcVerificationErrorList),
    InvalidInput(ValidationErrors),
    EmailAlreadyConfirmed,
    EmailConfirmationResent,
    Success,
}

pub async fn create(
    db_pool: PgPool,
    env: &Env,
    create_user: SignUpInput,
) -> anyhow::Result<CreateOutcome> {
    if !validate_frc(&create_user.frc_captcha_solution) {
        return Ok(CreateOutcome::PendingCaptcha);
    }

    if let Err(e) = verify_frc_solution(
        &create_user.frc_captcha_solution,
        &env.frc_sitekey,
        &env.frc_apikey,
    )
    .await
    {
        return Ok(CreateOutcome::InvalidCaptcha(e));
    }

    if let Err(e) = create_user.validate() {
        return Ok(CreateOutcome::InvalidInput(e));
    }

    let email_secrets = EmailSecrets {
        smtp_username: &env.smtp_username,
        smtp_host: &env.smtp_host,
        smtp_key: &env.smtp_key,
        mail_from: &env.mail_from,
    };

    let Some(expiration_date) = now_plus_24_hours() else {
        bail!("error adding 24 hours to current time");
    };
    let verification_token = generate_verification_token();

    // TODO: change this to not use a transaction, and instead insert the table without depending
    // on the emails being sent
    // create a email_sent_at kind of column, with a task that attempts to send it
    let mut transaction = db_pool.begin().await?;

    if let Some(user) =
        crate::queries::user::user_state_for_signup(&db_pool, &create_user.email).await?
    {
        let verification_token = generate_verification_token();
        crate::queries::user::set_email_prereq(
            &mut *transaction,
            &verification_token,
            expiration_date,
            user.id,
        )
        .await?;

        if user.verified {
            send_forgot_password_mail(&email_secrets, &create_user.email, &verification_token)?;
            transaction.commit().await?;
            return Ok(CreateOutcome::EmailAlreadyConfirmed);
        }

        send_sign_up_confirmation_mail(&email_secrets, &create_user.email, &verification_token)?;
        transaction.commit().await?;
        return Ok(CreateOutcome::EmailConfirmationResent);
    }

    let create_user_params = crate::queries::user::CreateParams {
        name: &create_user.username,
        email: &create_user.email,
        hashed_pass: &generate_hash(&create_user.password),
        verification_token: &verification_token,
        expiration_date,
    };

    crate::queries::user::create(&mut *transaction, create_user_params)
        .await
        .map(|c| {
            if c.rows_affected() > 1 {
                tracing::error!("i really need a macro that cancels the transaction");
            }
        })?;

    send_sign_up_confirmation_mail(&email_secrets, &create_user.email, &verification_token)?;

    transaction.commit().await?;
    Ok(CreateOutcome::Success)
}
