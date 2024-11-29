use sqlx::PgExecutor;
use time::OffsetDateTime;

pub struct CreateParams<'a> {
    pub name: &'a str,
    pub email: &'a str,
    pub hashed_pass: &'a str,
    pub verification_token: &'a str,
    pub expiration_date: OffsetDateTime,
}

pub async fn create(
    conn: impl PgExecutor<'_>,
    p: CreateParams<'_>,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO users (username, email, password, verification_code, code_expires_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        p.name,
        p.email,
        p.hashed_pass,
        p.verification_token,
        p.expiration_date
    )
    .execute(conn)
    .await
}

pub struct ForSignup {
    pub id: i32,
    pub verified: bool,
}

pub async fn user_state_for_signup(
    conn: impl PgExecutor<'_>,
    email: &str,
) -> Result<Option<ForSignup>, sqlx::Error> {
    sqlx::query_as!(
        ForSignup,
        r#"
        select id, verified from users where email = $1
        "#,
        email
    )
    .fetch_optional(conn)
    .await
}

pub async fn set_email_prereq(
    conn: impl PgExecutor<'_>,
    verification_code: &str,
    expires_at: OffsetDateTime,
    user_id: i32,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"
        update users set verification_code = $1, code_expires_at = $2
        where id = $3
        "#,
        verification_code,
        expires_at,
        user_id
    )
    .execute(conn)
    .await
}

pub async fn set_otp_secret(
    conn: impl PgExecutor<'_>,
    user_id: i32,
    secret: &totp_rs::Secret,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE users SET otp_secret = $1
        WHERE id = $2
        "#,
        secret.to_encoded().to_string(),
        user_id
    )
    .execute(conn)
    .await
}
