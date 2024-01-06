use askama_axum::IntoResponse;
use axum::{http::StatusCode, Json};
use sqlx::{Pool, Postgres};

use crate::{
    auth::AuthSession,
    schema::{Expense, ExpenseType, UpdateExpense},
};

pub async fn get_expenses(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let user_id = auth_session.user.expect("User should be authenticated").id;
    match sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
        FROM expenses
        WHERE user_id = $1
        ORDER BY date ASC"#,
        user_id
    )
    .fetch_all(db_pool)
    .await
    {
        Ok(expenses) => Ok((StatusCode::CREATED, Json(expenses))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn create_expense(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    Json(create_expense): Json<UpdateExpense>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let user_id = auth_session.user.expect("User should be authenticated").id;
    match sqlx::query_as!(
        Expense,
        r#"
        INSERT INTO expenses
        (description, price, expense_type, is_essencial, date, user_id)
        VALUES ($1, $2, $3 :: expense_type, $4, $5, $6)
        RETURNING id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
        "#,
        create_expense.description,
        create_expense.price,
        create_expense.expense_type as Option<ExpenseType>,
        create_expense.is_essencial,
        create_expense.date,
        user_id
    )
    .fetch_one(db_pool)
    .await
    {
        Ok(expense) => Ok((StatusCode::CREATED, Json(expense))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
