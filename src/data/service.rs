#![allow(unused_imports)]

use askama_axum::IntoResponse;
use axum::{http::StatusCode, Json};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    schema::{Expense, ExpenseCategory, UpdateExpense},
};

pub async fn get_expenses(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let user_id = auth_session.user.expect("User should be authenticated").id;
    match sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, category as "category: ExpenseCategory", is_essential, date, uuid
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

// pub async fn create_expense(
//     auth_session: AuthSession,
//     db_pool: &Pool<Postgres>,
//     Json(create_expense): Json<UpdateExpense>,
// ) -> Result<impl IntoResponse, impl IntoResponse> {
//     let user_id = auth_session.user.expect("User should be authenticated").id;
//     match sqlx::query_as!(
//         Expense,
//         r#"
//         INSERT INTO expenses
//         (description, price, category, is_essential, date, uuid, user_id)
//         VALUES ($1, $2, $3 :: expense_category, $4, $5, $6, $7)
//         RETURNING id, description, price, category as "category: ExpenseCategory", is_essential, date, uuid
//         "#,
//         create_expense.description,
//         create_expense.price,
//         create_expense.category as Option<ExpenseCategory>,
//         create_expense.is_essential,
//         create_expense.date,
//         Uuid::new_v4(),
//         user_id
//     )
//     .fetch_one(db_pool)
//     .await
//     {
//         Ok(expense) => Ok((StatusCode::CREATED, Json(expense))),
//         Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
//     }
// }
