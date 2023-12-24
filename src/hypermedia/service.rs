use crate::{
    constant::{EDITABLE_TABLE_ROW, TABLE_ROW},
    schema::{Expense, ExpenseType, GetExpense, UpdateExpense},
    util::{get_first_day_from_month_or_none, get_last_day_from_month_or_none},
};

use askama_axum::IntoResponse;
use axum::{
    extract::{Path, Query},
    response::Html,
    Json,
};
use sqlx::{Pool, Postgres};

pub async fn get_expenses(
    db_pool: &Pool<Postgres>,
    Query(get_expense_input): Query<GetExpense>,
) -> impl IntoResponse {
    let expenses = sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
        FROM expenses
        WHERE ((date >= $1) OR ($1 IS NULL))
        AND ((date <= $2) OR ($2 IS NULL))
        ORDER BY date ASC"#,
        get_first_day_from_month_or_none(get_expense_input.month.clone()),
        get_last_day_from_month_or_none(get_expense_input.month)
    )
    .fetch_all(db_pool)
    .await
    .unwrap();

    tracing::info!("expenses: {:?}", expenses);

    Html(
        expenses
            .iter()
            .map(|expense| {
                format!(
                    TABLE_ROW!(),
                    expense.date,
                    expense.description,
                    expense.price,
                    expense.expense_type,
                    expense.is_essencial,
                    expense.id
                )
            })
            .collect::<Vec<String>>()
            .join("\n"),
    )
}

pub async fn edit_expense(db_pool: &Pool<Postgres>, Path(id): Path<i32>) -> impl IntoResponse {
    let expense = sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
        FROM expenses WHERE id = $1"#,
        id
    )
        .fetch_one(db_pool)
        .await
        .unwrap();

    Html(format!(
        EDITABLE_TABLE_ROW!(),
        expense.id,
        expense.date,
        expense.description,
        expense.price,
        if expense.is_essencial { "checked" } else { "" },
        expense.id,
        expense.id
    ))
}

pub async fn get_expense(db_pool: &Pool<Postgres>, Path(id): Path<i32>) -> impl IntoResponse {
    let expense = sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
        FROM expenses WHERE id = $1"#,
        id
    )
    .fetch_one(db_pool)
    .await
    .unwrap();

    Html(format!(
        TABLE_ROW!(),
        expense.date,
        expense.description,
        expense.price,
        expense.expense_type,
        expense.is_essencial,
        expense.id
    ))
}

pub async fn update_expense(
    db_pool: &Pool<Postgres>,
    Path(id): Path<i32>,
    Json(update_expense): Json<UpdateExpense>,
) -> impl IntoResponse {
    let expense = sqlx::query_as!(
        Expense,
        r#"
        UPDATE expenses SET
            description = COALESCE($1, description),
            price = COALESCE($2, price),
            expense_type = COALESCE($3 :: expense_type, expense_type),
            is_essencial = COALESCE($4, is_essencial),
            date = COALESCE($5, date)
        WHERE id = $6
        RETURNING id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
        "#,
        update_expense.description,
        update_expense.price,
        update_expense.expense_type as Option<ExpenseType>,
        update_expense.is_essencial,
        update_expense.date,
        id
    )
    .fetch_one(db_pool)
    .await
    .unwrap();

    Html(format!(
        TABLE_ROW!(),
        expense.date,
        expense.description,
        expense.price,
        expense.expense_type,
        expense.is_essencial,
        expense.id
    ))
}
