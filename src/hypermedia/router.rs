use crate::{
    auth::{AuthSession, LoginCredentials, SignUpCredentials},
    schema::{GetExpense, UpdateExpense},
    AppState, ExpensesTemplate, SignInTemplate,
};
use std::sync::Arc;

use askama_axum::IntoResponse;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};

use super::validation_service::EmailInput;

// VALIDATION
pub fn validation_router() -> Router<Arc<AppState>> {
    Router::new().route("/validate/email", get(validate_email))
}

async fn validate_email(Query(input_email): Query<EmailInput>) -> impl IntoResponse {
    super::validation_service::validate_email(input_email).await
}

// AUTH
pub fn auth_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth", get(auth_index))
        .route("/signin", get(signin_tab).post(signin))
        .route("/signup", get(signup_tab).post(signup))
}

async fn auth_index() -> impl IntoResponse {
    SignInTemplate {}
}

async fn signin_tab() -> impl IntoResponse {
    super::auth_service::signin_tab().await
}

async fn signin(
    auth_session: AuthSession,
    Json(signin_input): Json<LoginCredentials>,
) -> impl IntoResponse {
    super::auth_service::signin(Json(signin_input), auth_session).await
}

async fn signup_tab() -> impl IntoResponse {
    super::auth_service::signup_tab().await
}

async fn signup(
    State(shared_state): State<Arc<AppState>>,
    Json(signup_input): Json<SignUpCredentials>,
) -> impl IntoResponse {
    super::auth_service::signup(&shared_state.pool, Json(signup_input)).await
}

// EXPENSES
pub fn expenses_router() -> Router<Arc<AppState>> {
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
    super::expenses_service::get_expenses(
        auth_session,
        &shared_state.pool,
        Query(get_expense_input),
    )
    .await
}

async fn edit_expense(
    auth_session: AuthSession,
    Path(id): Path<i32>,
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    super::expenses_service::edit_expense(auth_session, &shared_state.pool, Path(id)).await
}

async fn get_expense(
    auth_session: AuthSession,
    Path(id): Path<i32>,
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    super::expenses_service::get_expense(auth_session, &shared_state.pool, Path(id)).await
}

async fn update_expense(
    auth_session: AuthSession,
    Path(id): Path<i32>,
    State(shared_state): State<Arc<AppState>>,
    Json(update_expense): Json<UpdateExpense>,
) -> impl IntoResponse {
    super::expenses_service::update_expense(
        auth_session,
        &shared_state.pool,
        Path(id),
        Json(update_expense),
    )
    .await
}

async fn insert_expense(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Json(create_expense): Json<UpdateExpense>,
) -> impl IntoResponse {
    super::expenses_service::insert_expense(auth_session, &shared_state.pool, Json(create_expense))
        .await
}

async fn expenses_plots(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Query(get_expense_input): Query<GetExpense>,
) -> impl IntoResponse {
    super::expenses_service::expenses_plots(
        auth_session,
        &shared_state.pool,
        Query(get_expense_input),
    )
    .await
}
