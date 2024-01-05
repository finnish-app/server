use crate::{
    auth::{AuthSession, LoginCredentials},
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
use validator::Validate;

pub fn hypermedia_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(expenses_index))
        .route("/expenses", get(get_expenses).post(insert_expense))
        .route("/expenses/:id/edit", get(edit_expense))
        .route("/expenses/:id", get(get_expense).put(update_expense))
        .route("/expenses/plots", get(expenses_plots))
}

pub mod auth {
    use super::*;
    pub fn auth_router() -> Router<Arc<AppState>> {
        Router::new()
            .route("/auth", get(auth_index))
            .route("/signin", get(signin_tab).post(signin))
            .route("/signup", get(signup_tab).post(signup))
    }
}

pub mod util {
    use super::*;
    use axum::response::Html;
    use serde::Deserialize;

    #[derive(Deserialize, Validate)]
    struct EmailInput {
        #[validate(email)]
        email: String,
    }

    pub fn util_router() -> Router<Arc<AppState>> {
        Router::new().route("/validate/email", get(validate_email_input))
    }

    async fn validate_email_input(Query(input): Query<EmailInput>) -> impl IntoResponse {
        match input.validate() {
            Ok(_) => Html(format!(
                "
                <div hx-target=\"this\" hx-swap=\"outerHTML\">
                    <label for=\"email\">Email</label>
                    <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
                    <input
                      type=\"email\"
                      name=\"email\"
                      placeholder=\"email@server.com\"
                      aria-label=\"Email\"
                      aria-invalid=\"false\"
                      autocomplete=\"email\"
                      hx-get=\"/validate/email\"
                      hx-sync=\"closest form:abort\"
                      hx-indicator=\"#ind\"
                      value=\"{}\"
                      required
                    />
                </div>
                ",
                input.email
            ))
            .into_response(),
            Err(_) => Html(format!(
                "
                <div hx-target=\"this\" hx-swap=\"outerHTML\">
                    <label for=\"email\">Email</label>
                    <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
                    <input
                      type=\"email\"
                      name=\"email\"
                      placeholder=\"email@server.com\"
                      aria-label=\"Email\"
                      aria-invalid=\"true\"
                      autocomplete=\"email\"
                      hx-get=\"/validate/email\"
                      hx-sync=\"closest form:abort\"
                      hx-indicator=\"#ind\"
                      value=\"{}\"
                      required
                    />
                </div>
                ",
                input.email
            ))
            .into_response(),
        }
    }
}

async fn auth_index() -> impl IntoResponse {
    SignInTemplate {}
}

async fn signin_tab() -> impl IntoResponse {
    super::service::signin_tab().await
}

async fn signin(
    auth_session: AuthSession,
    Json(signin_input): Json<LoginCredentials>,
) -> impl IntoResponse {
    super::service::signin(Json(signin_input), auth_session).await
}

async fn signup_tab() -> impl IntoResponse {
    super::service::signup_tab().await
}

async fn signup(
    State(shared_state): State<Arc<AppState>>,
    Json(signup_input): Json<LoginCredentials>,
) -> impl IntoResponse {
    super::service::signup(&shared_state.pool, Json(signup_input)).await
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
    super::service::get_expenses(auth_session, &shared_state.pool, Query(get_expense_input)).await
}

async fn edit_expense(
    auth_session: AuthSession,
    Path(id): Path<i32>,
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    super::service::edit_expense(auth_session, &shared_state.pool, Path(id)).await
}

async fn get_expense(
    auth_session: AuthSession,
    Path(id): Path<i32>,
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    super::service::get_expense(auth_session, &shared_state.pool, Path(id)).await
}

async fn update_expense(
    auth_session: AuthSession,
    Path(id): Path<i32>,
    State(shared_state): State<Arc<AppState>>,
    Json(update_expense): Json<UpdateExpense>,
) -> impl IntoResponse {
    super::service::update_expense(
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
    super::service::insert_expense(auth_session, &shared_state.pool, Json(create_expense)).await
}

async fn expenses_plots(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Query(get_expense_input): Query<GetExpense>,
) -> impl IntoResponse {
    super::service::expenses_plots(auth_session, &shared_state.pool, Query(get_expense_input)).await
}
