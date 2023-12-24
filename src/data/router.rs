use axum::{routing::get, Json};
use std::sync::Arc;

use askama_axum::IntoResponse;
use axum::{extract::State, Router};

use crate::{schema::UpdateExpense, AppState};

pub fn data_router() -> Router<Arc<AppState>> {
    Router::new().route("/api/expenses", get(get_expenses).post(create_expense))
}

pub async fn get_expenses(
    State(shared_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    super::service::get_expenses(&shared_state.pool).await
}

pub async fn create_expense(
    State(shared_state): State<Arc<AppState>>,
    Json(create_expense): Json<UpdateExpense>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    super::service::create_expense(&shared_state.pool, Json(create_expense)).await
}
