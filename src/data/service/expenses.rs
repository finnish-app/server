#![allow(unused_imports)]

use askama_axum::IntoResponse;
use axum::{http::StatusCode, Json};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    schema::{Expense, ExpenseCategory, GetExpense, UpdateExpense},
    util::{get_first_day_from_month_or_none, get_last_day_from_month_or_none},
};

pub async fn get_expenses(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    get_expense_input: GetExpense,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let user_id = if let Some(user) = auth_session.user {
        tracing::info!("User logged in");
        user.id
    } else {
        tracing::error!("User not logged in");
        return Err((StatusCode::UNAUTHORIZED, String::new()));
    };

    let first_day_of_month = match get_first_day_from_month_or_none(get_expense_input.month.clone())
    {
        Ok(date) => date,
        Err(e) => {
            tracing::error!("Error getting first day of month: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, String::new()));
        }
    };
    let last_day_of_month = match get_last_day_from_month_or_none(get_expense_input.month) {
        Ok(date) => date,
        Err(e) => {
            tracing::error!("Error getting last day of month: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, String::new()));
        }
    };
    match sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, category as "category: ExpenseCategory", is_essential, date, uuid
        FROM expenses
        WHERE ((date >= $1) OR ($1 IS NULL))
        AND ((date <= $2) OR ($2 IS NULL))
        AND user_id = $3
        ORDER BY date ASC"#,
        first_day_of_month,
        last_day_of_month,
        user_id
    )
    .fetch_all(db_pool)
    .await
    {
        Ok(expenses) => Ok((StatusCode::CREATED, Json(expenses))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn insert_expense(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    create_expense: UpdateExpense,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let user_id = if let Some(user) = auth_session.user {
        tracing::info!("User logged in");
        user.id
    } else {
        tracing::error!("User not logged in");
        return Err((StatusCode::UNAUTHORIZED, String::new()));
    };

    match sqlx::query_as!(
        Expense,
        r#"
        INSERT INTO expenses (description, price, category, is_essential, date, uuid, user_id)
        VALUES ($1, $2, $3 :: expense_category, $4, $5, $6, $7)
        RETURNING id, description, price, category as "category: ExpenseCategory", is_essential, date, uuid
        "#,
        create_expense.description,
        create_expense.price,
        create_expense.category as Option<ExpenseCategory>,
        create_expense.is_essential,
        create_expense.date,
        Uuid::new_v4(),
        user_id
    )
    .fetch_one(db_pool)
    .await {
        Ok(expense) => Ok((StatusCode::CREATED, Json(expense))),
        Err(e) => {
            tracing::error!("Error inserting expense: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        },
    }
}
