use std::sync::Arc;

use askama_axum::IntoResponse;
use axum::extract::Path;
use axum::routing::get;
use axum::Router;
use axum::{extract::State, response::Html};

use crate::{
    schema::Expense,
    util::{EDITABLE_TABLE_ROW, TABLE_ROW},
    AppState, ExpensesTemplate,
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

pub async fn get_expenses(State(shared_state): State<Arc<AppState>>) -> impl IntoResponse {
    //    let expenses: Vec<Expense> = sqlx::query_as!(Expense, "SELECT * FROM expenses")
    //        .fetch_all(&shared_state.pool)
    //        .await
    //        .unwrap();
    let expenses: Vec<Expense> = sqlx::query_as("SELECT * FROM expenses")
        .fetch_all(&shared_state.pool)
        .await
        .unwrap();

    Html(
        expenses
            .iter()
            .map(|expense| {
                format!(
                    TABLE_ROW!(),
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
        expense.description, expense.price, expense.expense_type, expense.is_essencial, expense.id
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
        expense.description, expense.price, expense.expense_type, expense.is_essencial, expense.id
    ))
}
