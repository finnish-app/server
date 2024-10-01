use sqlx::{postgres::PgQueryResult, Error, Pool, Postgres};
use time::OffsetDateTime;
use uuid::Uuid;

pub async fn add_item(
    db_pool: &Pool<Postgres>,
    user_id: i32,
    item_id: Uuid,
    now: OffsetDateTime,
) -> Result<PgQueryResult, Error> {
    sqlx::query!(
        r#"INSERT INTO pluggy_items (user_id, external_item_id, created_at)
    VALUES ($1, $2, $3)"#,
        user_id,
        item_id,
        now
    )
    .execute(db_pool)
    .await
}
