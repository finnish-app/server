use axum::routing::get;
use std::sync::Arc;

use askama_axum::IntoResponse;
use axum::{extract::State, http::StatusCode, Json, Router};

use crate::{schema::Expense, AppState};

pub fn data_router() -> Router<Arc<AppState>> {
    Router::new().route("/api/expenses", get(get_expenses))
}

pub async fn get_expenses(
    State(shared_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Expense>("SELECT * FROM expenses")
        .fetch_all(&shared_state.pool)
        .await
    {
        Ok(expenses) => Ok((StatusCode::CREATED, Json(expenses))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
