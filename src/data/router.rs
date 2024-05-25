#![allow(unused_imports)]

use axum::{extract::Query, routing::get, Json};
use std::sync::Arc;

use askama_axum::IntoResponse;
use axum::{extract::State, Router};

use crate::{
    auth::AuthSession,
    schema::{GetExpense, UpdateExpense},
    AppState,
};

pub fn data_router() -> Router<Arc<AppState>> {
    Router::new().route("/api/expenses", get(get_expenses)) //.post(insert_expense))
}

async fn get_expenses(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Query(get_expenses_input): Query<GetExpense>,
) -> impl IntoResponse {
    crate::data::service::expenses::get_expenses(
        auth_session,
        &shared_state.pool,
        get_expenses_input,
    )
    .await
}

async fn insert_expense(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Json(create_expense): Json<UpdateExpense>,
) -> impl IntoResponse {
    crate::data::service::expenses::insert_expense(
        auth_session,
        &shared_state.pool,
        create_expense,
    )
    .await
}
