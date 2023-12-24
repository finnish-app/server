use crate::{
    schema::{GetExpense, UpdateExpense},
    AppState, ExpensesTemplate,
};
use std::sync::Arc;

use askama_axum::IntoResponse;
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
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
    super::service::get_expenses(&shared_state.pool, Query(get_expense_input)).await
}

pub async fn edit_expense(
    Path(id): Path<i32>,
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    super::service::edit_expense(&shared_state.pool, Path(id)).await
}

pub async fn get_expense(
    Path(id): Path<i32>,
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    super::service::get_expense(&shared_state.pool, Path(id)).await
}

pub async fn update_expense(
    Path(id): Path<i32>,
    State(shared_state): State<Arc<AppState>>,
    Json(update_expense): Json<UpdateExpense>,
) -> impl IntoResponse {
    super::service::update_expense(&shared_state.pool, Path(id), Json(update_expense)).await
}
