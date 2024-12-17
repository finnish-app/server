use sqlx::PgExecutor;
use time::OffsetDateTime;
use uuid::Uuid;

pub async fn create(
    conn: impl PgExecutor<'_>,
    user_id: i32,
    item_id: Uuid,
    now: OffsetDateTime,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO pluggy_items (user_id, external_item_id, created_at, last_updated_at)
    VALUES ($1, $2, $3, $4)"#,
        user_id,
        item_id,
        now,
        now
    )
    .execute(conn)
    .await
}

pub struct UserPluggyItem {
    pub external_item_id: Uuid,
}

pub async fn find_latest_for_user(
    conn: impl PgExecutor<'_>,
    user_id: i32,
) -> Result<Option<UserPluggyItem>, sqlx::Error> {
    sqlx::query_as!(
        UserPluggyItem,
        r#"select external_item_id from pluggy_items
        where user_id = $1
        order by created_at desc
        "#,
        user_id,
    )
    .fetch_optional(conn)
    .await
}
