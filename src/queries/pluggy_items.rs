use sqlx::PgExecutor;
use time::OffsetDateTime;
use uuid::Uuid;

pub async fn create(
    conn: impl PgExecutor<'_>,
    user_id: i32,
    item_id: Uuid,
    now: OffsetDateTime,
    connector_name: String,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO pluggy_items (user_id, external_item_id, created_at, last_updated_at, connector_name)
    VALUES ($1, $2, $3, $4, $5)"#,
        user_id,
        item_id,
        now,
        now,
        connector_name
    )
    .execute(conn)
    .await
}

pub struct ItemFound {
    pub id: i32,
}

pub async fn find_item(
    conn: impl PgExecutor<'_>,
    user_id: i32,
    external_item_id: Uuid,
) -> Result<Option<ItemFound>, sqlx::Error> {
    sqlx::query_as!(
        ItemFound,
        r#"
        SELECT id
        FROM pluggy_items
        WHERE user_id = $1
        AND external_item_id = $2
        "#,
        user_id,
        external_item_id,
    )
    .fetch_optional(conn)
    .await
}

pub struct UserPluggyItem {
    pub external_item_id: Uuid,
    pub connector_name: String,
}

pub async fn find_latests_for_user(
    conn: impl PgExecutor<'_>,
    user_id: i32,
) -> Result<Vec<UserPluggyItem>, sqlx::Error> {
    sqlx::query_as!(
        UserPluggyItem,
        r#"
        SELECT DISTINCT ON (connector_name)
        external_item_id,
        connector_name
        FROM
            pluggy_items
        WHERE
            user_id = $1
        ORDER BY
            connector_name,
            created_at DESC;
        "#,
        user_id,
    )
    .fetch_all(conn)
    .await
}

pub async fn updated(
    conn: impl PgExecutor<'_>,
    user_id: i32,
    item_id: i32,
    now: OffsetDateTime,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE pluggy_items SET last_updated_at = $1
        WHERE id = $2 AND user_id = $3
        "#,
        now,
        user_id,
        item_id,
    )
    .execute(conn)
    .await
}
