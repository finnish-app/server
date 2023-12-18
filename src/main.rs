use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use shuttle_runtime::CustomError;
use sqlx::{FromRow, PgPool};
use tower_http::services::ServeDir;

struct AppState {
    pool: PgPool,
}

#[derive(FromRow, Serialize)]
struct Expense {
    id: i32,
    amount: f32,
}

#[shuttle_runtime::main]
async fn axum(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(CustomError::new)?;

    let shared_state = Arc::new(AppState { pool });
    let router = Router::new()
        .route("/", get(expenses_index))
        .route("/expenses", get(get_expenses))
        .route("/hello", get(hello_world))
        .route("/greet/:name", get(greet))
        .route("/whatever", get(whatever))
        .nest_service("/static", ServeDir::new("./css"))
        .with_state(shared_state);

    Ok(router.into())
}

async fn whatever(
    State(shared_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as!(Expense, "SELECT * FROM expenses")
        .fetch_all(&shared_state.pool)
        .await
    {
        Ok(expenses) => Ok((StatusCode::CREATED, Json(expenses))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate {
    name: String,
}

async fn greet(Path(name): Path<String>) -> impl IntoResponse {
    HelloTemplate { name }
}

#[derive(Template)]
#[template(path = "expenses.html")]
struct ExpensesTemplate {}

async fn expenses_index() -> impl IntoResponse {
    ExpensesTemplate {}
}

async fn get_expenses(State(shared_state): State<Arc<AppState>>) -> impl IntoResponse {
    let expenses: Vec<Expense> = sqlx::query_as!(Expense, "SELECT * FROM expenses")
        .fetch_all(&shared_state.pool)
        .await
        .unwrap();

    Html(
        expenses
            .iter()
            .map(|expense| {
                format!(
                    "<tr><td>{}</td><td>{}</td></tr>",
                    expense.id, expense.amount
                )
            })
            .collect::<Vec<String>>()
            .join("\n"),
    )
}
