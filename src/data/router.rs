#![allow(unused_imports)]

use axum::{routing::get, Json};
use std::sync::Arc;

use askama_axum::IntoResponse;
use axum::{extract::State, Router};

use crate::{auth::AuthSession, schema::UpdateExpense, AppState};

pub fn data_router() -> Router<Arc<AppState>> {
    Router::new().route("/api/expenses", get(get_expenses))//.post(insert_expense))
}

async fn get_expenses(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    super::service::get_expenses(auth_session, &shared_state.pool).await
}

// async fn insert_expense(
//     auth_session: AuthSession,
//     State(shared_state): State<Arc<AppState>>,
//     Json(create_expense): Json<UpdateExpense>,
// ) -> Result<impl IntoResponse, impl IntoResponse> {
//     super::service::create_expense(auth_session, &shared_state.pool, Json(create_expense)).await
// }
