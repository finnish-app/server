use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

pub async fn add_item(
    db_pool: PgPool,
    user_id: i32,
    item_id: Uuid,
    connector_name: String,
    now: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    crate::queries::pluggy_items::create(&db_pool, user_id, item_id, now, connector_name)
        .await
        .map(|c| {
            if c.rows_affected() > 1 {
                tracing::error!("i really need a macro that cancels the transaction");
            }
            Ok(())
        })?
}
