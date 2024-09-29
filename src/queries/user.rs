use chrono::{DateTime, Utc};
use sqlx::PgExecutor;

pub struct CreateParams<'a> {
    pub name: &'a str,
    pub email: &'a str,
    pub hashed_pass: &'a str,
    pub verification_token: &'a str,
    pub expiration_date: DateTime<Utc>,
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

pub async fn is_email_taken(
    conn: impl PgExecutor<'_>,
    email: &str,
) -> Result<Option<bool>, sqlx::Error> {
    let record = sqlx::query!(
        r#"
    select exists (select 1 from users where email = $1)
    "#,
        email
    )
    .fetch_one(conn)
    .await?;

    Ok(record.exists)
}
