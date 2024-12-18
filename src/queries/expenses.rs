use serde::Serialize;
use sqlx::{PgExecutor, PgPool};
use time::{Date, OffsetDateTime};
use uuid::Uuid;

use crate::schema::ExpenseCategory;

pub struct CreateParams {
    pub description: String,
    pub price: f32,
    pub category: ExpenseCategory,
    pub is_essential: Option<bool>,
    pub date: Date,
    pub now: OffsetDateTime,
}

pub async fn create(
    conn: impl PgExecutor<'_>,
    user_id: i32,
    p: CreateParams,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO expenses (description, price, category, is_essential, date, uuid, user_id, created_at)
        VALUES ($1, $2, $3 :: expense_category, $4, $5, $6, $7, $8)
        "#,
        p.description,
        p.price,
        p.category as ExpenseCategory,
        p.is_essential.unwrap_or_default(),
        p.date,
        Uuid::new_v4(),
        user_id,
        p.now,
    )
    .execute(conn)
    .await
}

pub struct CreateParamsFromPluggy {
    pub description: String,
    pub price: f32,
    pub bank: Option<String>,
    pub date: Date,
    pub now: OffsetDateTime,
    pub external_account_id: Uuid,
    pub external_id: Uuid,
    pub external_created_at: Option<OffsetDateTime>,
    pub is_essential: bool,
    pub external_category: Option<String>,
}

pub async fn insert_from_pluggy(
    conn: impl PgExecutor<'_>,
    user_id: i32,
    p: CreateParamsFromPluggy,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO expenses
        (description, price, bank_source, external_account_id, external_id, date, uuid, is_essential, user_id, created_at, external_created_at, external_category)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
        p.description,
        p.price,
        p.bank,
        p.external_account_id,
        p.external_id,
        p.date,
        Uuid::new_v4(),
        p.is_essential,
        user_id,
        p.now,
        p.external_created_at,
        p.external_category,
    )
    .execute(conn)
    .await
}

#[derive(Serialize)]
/// Expense is a struct with the fields of an expense.
pub struct Expense {
    pub id: i32,
    pub description: String,
    pub price: f32,
    pub category: Option<ExpenseCategory>,
    pub is_essential: bool,
    pub date: Date,
    pub uuid: Uuid,
    pub bank_source: Option<String>,
}

pub async fn list_for_user_in_period(
    conn: impl PgExecutor<'_>,
    user_id: i32,
    from: Option<Date>,
    to: Option<Date>,
) -> Result<Vec<Expense>, sqlx::Error> {
    sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, category as "category: ExpenseCategory", is_essential, date, uuid, bank_source
        FROM expenses
        WHERE ((date >= $1) OR ($1 IS NULL))
        AND ((date <= $2) OR ($2 IS NULL))
        AND user_id = $3
        and deleted_at is null"#,
        from,
        to,
        user_id
    )
    .fetch_all(conn)
    .await
}

pub async fn find_active_for_user(
    db_pool: &PgPool,
    user_id: i32,
    expense_uuid: Uuid,
) -> Result<Expense, sqlx::Error> {
    sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, category as "category: ExpenseCategory", is_essential, date, uuid, bank_source
        FROM expenses
        WHERE uuid = $1 AND user_id = $2
        and deleted_at is null
        "#,
        expense_uuid,
        user_id
    )
    .fetch_one(db_pool)
    .await
}

pub async fn exists_active(
    db_pool: &PgPool,
    user_id: i32,
    expense_uuid: Uuid,
) -> Result<Option<bool>, sqlx::Error> {
    let record = sqlx::query!(
        r#"
        select exists(select 1 from expenses where uuid = $1 AND user_id = $2)
        "#,
        expense_uuid,
        user_id
    )
    .fetch_one(db_pool)
    .await?;

    Ok(record.exists)
}

pub struct UpdateParams {
    pub description: Option<String>,
    pub price: Option<f32>,
    pub category: Option<ExpenseCategory>,
    pub is_essential: Option<bool>,
    pub date: Option<Date>,
}

pub async fn update(
    db_pool: &PgPool,
    user_id: i32,
    expense_uuid: Uuid,
    p: UpdateParams,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE expenses SET
            description = COALESCE($1, description),
            price = COALESCE($2, price),
            category = COALESCE($3 :: expense_category, category),
            is_essential = COALESCE($4, is_essential),
            date = COALESCE($5, date)
        WHERE uuid = $6 AND user_id = $7
        and deleted_at is null
        "#,
        p.description,
        p.price,
        p.category as Option<ExpenseCategory>,
        p.is_essential,
        p.date,
        expense_uuid,
        user_id
    )
    .execute(db_pool)
    .await
}

pub async fn update_for_site(
    db_pool: &PgPool,
    user_id: i32,
    expense_uuid: Uuid,
    p: UpdateParams,
) -> Result<Expense, sqlx::Error> {
    sqlx::query_as!(
        Expense,
        r#"
        UPDATE expenses SET
            description = COALESCE($1, description),
            price = COALESCE($2, price),
            category = COALESCE($3 :: expense_category, category),
            is_essential = COALESCE($4, is_essential),
            date = COALESCE($5, date)
        WHERE uuid = $6 AND user_id = $7
        and deleted_at is null
        RETURNING id, description, price, category as "category: ExpenseCategory", is_essential, date, uuid, bank_source
        "#,
        p.description,
        p.price,
        p.category as Option<ExpenseCategory>,
        p.is_essential,
        p.date,
        expense_uuid,
        user_id
    )
    .fetch_one(db_pool)
    .await
}

pub async fn delete(
    db_pool: &PgPool,
    user_id: i32,
    expense_uuid: Uuid,
    now: OffsetDateTime,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"
        update expenses
        set deleted_at = $1
        WHERE uuid = $2 AND user_id = $3
        "#,
        now,
        expense_uuid,
        user_id,
    )
    .execute(db_pool)
    .await
}

pub struct MostRecentTimestamps {
    pub date: Date,
    pub external_created_at: Option<OffsetDateTime>,
}

pub async fn most_recent_for_account(
    conn: impl PgExecutor<'_>,
    user_id: i32,
    external_account_id: Uuid,
) -> Result<Option<MostRecentTimestamps>, sqlx::Error> {
    sqlx::query_as!(
        MostRecentTimestamps,
        r#"
        SELECT date, external_created_at
        FROM expenses
        WHERE external_account_id = $1
        AND user_id = $2
        order by external_created_at desc, date desc
        limit 1
        "#,
        external_account_id,
        user_id,
    )
    .fetch_optional(conn)
    .await
}

pub struct UncategorizedTransactions {
    pub id: i32,
    pub description: String,
    pub price: f32,
    pub external_category: Option<String>,
}

pub async fn list_uncategorized(
    conn: impl PgExecutor<'_>,
) -> Result<Vec<UncategorizedTransactions>, sqlx::Error> {
    sqlx::query_as!(
        UncategorizedTransactions,
        r#"
        SELECT id, description, price, external_category
        FROM expenses
        WHERE category IS NULL
        "#
    )
    .fetch_all(conn)
    .await
}

pub async fn update_category(
    conn: impl PgExecutor<'_>,
    expense_id: i32,
    category: ExpenseCategory,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"
        update expenses
        set category = $1
        WHERE id = $2
        "#,
        category as ExpenseCategory,
        expense_id,
    )
    .execute(conn)
    .await
}
