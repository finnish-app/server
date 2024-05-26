use askama_axum::IntoResponse;
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    schema::{GetExpense, UpdateExpense},
    AppState,
};

pub fn data_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/expenses", get(get_expenses).post(insert_expense))
        .route(
            "/api/:uuid",
            get(get_expense).put(update_expense).delete(delete_expense),
        )
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
    crate::data::service::expenses::insert_expense(auth_session, &shared_state.pool, create_expense)
        .await
}

async fn get_expense(
    auth_session: AuthSession,
    Path(uuid): Path<Uuid>,
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    crate::data::service::expenses::get_expense(auth_session, &shared_state.pool, uuid).await
}

async fn update_expense(
    auth_session: AuthSession,
    Path(uuid): Path<Uuid>,
    State(shared_state): State<Arc<AppState>>,
    Json(update_expense): Json<UpdateExpense>,
) -> impl IntoResponse {
    crate::data::service::expenses::update_expense(
        auth_session,
        &shared_state.pool,
        uuid,
        update_expense,
    )
    .await
}

async fn delete_expense(
    auth_session: AuthSession,
    Path(uuid): Path<Uuid>,
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    crate::data::service::expenses::delete_expense(auth_session, &shared_state.pool, uuid).await
}
