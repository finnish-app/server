use crate::{
    auth::AuthSession,
    schema::{GetExpense, UpdateExpense},
    AppState, ExpensesTemplate,
};
use std::sync::Arc;

use askama_axum::IntoResponse;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(expenses_index))
        .route("/expenses", get(get_expenses).post(insert_expense))
        .route("/expenses/:id/edit", get(edit_expense))
        .route("/expenses/:id", get(get_expense).put(update_expense))
        .route("/expenses/plots", get(expenses_plots))
}

async fn expenses_index(auth_session: AuthSession) -> impl IntoResponse {
    match auth_session.user {
        Some(user) => ExpensesTemplate {
            username: &user.username,
            ..Default::default()
        }
        .into_response(),
        None => StatusCode::UNAUTHORIZED.into_response(),
    }
}

async fn get_expenses(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Query(get_expense_input): Query<GetExpense>,
) -> impl IntoResponse {
    crate::hypermedia::service::expenses::get_expenses(
        auth_session,
        &shared_state.pool,
        get_expense_input,
    )
    .await
}

async fn edit_expense(
    auth_session: AuthSession,
    Path(id): Path<i32>,
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    crate::hypermedia::service::expenses::edit_expense(auth_session, &shared_state.pool, id).await
}

async fn get_expense(
    auth_session: AuthSession,
    Path(id): Path<i32>,
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    crate::hypermedia::service::expenses::get_expense(auth_session, &shared_state.pool, id).await
}

async fn update_expense(
    auth_session: AuthSession,
    Path(id): Path<i32>,
    State(shared_state): State<Arc<AppState>>,
    Json(update_expense): Json<UpdateExpense>,
) -> impl IntoResponse {
    crate::hypermedia::service::expenses::update_expense(
        auth_session,
        &shared_state.pool,
        id,
        update_expense,
    )
    .await
}

async fn insert_expense(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Json(create_expense): Json<UpdateExpense>,
) -> impl IntoResponse {
    crate::hypermedia::service::expenses::insert_expense(
        auth_session,
        &shared_state.pool,
        create_expense,
    )
    .await
}

async fn expenses_plots(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Query(get_expense_input): Query<GetExpense>,
) -> impl IntoResponse {
    crate::hypermedia::service::expenses::expenses_plots(
        auth_session,
        &shared_state.pool,
        get_expense_input,
    )
    .await
}