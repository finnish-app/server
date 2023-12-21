use crate::{
    constant::{EDITABLE_TABLE_ROW, TABLE_ROW},
    schema::{Expense, ExpenseType, GetExpense},
    util::{get_first_day_from_month, get_last_day_from_month},
    AppState, ExpensesTemplate,
};
use std::sync::Arc;

use askama_axum::IntoResponse;
use axum::{
    extract::{Path, Query, State},
    response::Html,
    routing::get,
    Router,
};

pub fn hypermedia_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(expenses_index))
        .route("/expenses", get(get_expenses))
        .route("/expenses/:id/edit", get(edit_expense))
        .route("/expenses/:id", get(get_expense).put(update_expense))
}

pub async fn expenses_index() -> impl IntoResponse {
    ExpensesTemplate {
        ..Default::default()
    }
}

pub async fn get_expenses(
    State(shared_state): State<Arc<AppState>>,
    Query(get_expense_input): Query<GetExpense>,
) -> impl IntoResponse {
    let expenses: Vec<Expense> = match get_expense_input.month {
        Some(month) => {
            sqlx::query_as!(
                Expense,
                r#"SELECT id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
                FROM expenses WHERE date BETWEEN $1 AND $2 ORDER BY date ASC"#,
                get_first_day_from_month(month.clone() as u32 + 1),
                get_last_day_from_month(month as u32 + 1)
            )
            .fetch_all(&shared_state.pool)
            .await
            .unwrap()
        }
        None => {
            sqlx::query_as!(
                Expense,
                r#"SELECT id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
                FROM expenses ORDER BY date ASC"#
            )
                .fetch_all(&shared_state.pool)
                .await
                .unwrap()
        }
    };

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

pub async fn edit_expense(
    Path(id): Path<i32>,
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let expense: Expense = sqlx::query_as("SELECT * FROM expenses WHERE id = $1")
        .bind(id)
        .fetch_one(&shared_state.pool)
        .await
        .unwrap();

    Html(format!(
        EDITABLE_TABLE_ROW!(),
        expense.id,
        expense.date,
        expense.description,
        expense.price,
        expense.is_essencial,
        expense.id,
        expense.id
    ))
}

pub async fn get_expense(
    Path(id): Path<i32>,
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let expense: Expense = sqlx::query_as("SELECT * FROM expenses WHERE id = $1")
        .bind(id)
        .fetch_one(&shared_state.pool)
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
    Path(id): Path<i32>,
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let expense: Expense = sqlx::query_as("SELECT * FROM expenses WHERE id = $1")
        .bind(id)
        .fetch_one(&shared_state.pool)
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
