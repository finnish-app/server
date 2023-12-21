mod constant;
mod data;
mod data_router;
mod hypermedia_router;
mod schema;
mod util;

use crate::data::{Months, MonthsIter};
use std::sync::Arc;

use askama::Template;
use axum::Router;
use chrono::{Datelike, Month, Utc};
use shuttle_runtime::CustomError;
use sqlx::PgPool;
use strum::IntoEnumIterator;
use tower_http::services::ServeDir;

struct AppState {
    pool: PgPool,
}

#[shuttle_runtime::main]
async fn axum(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(CustomError::new)?;

    let shared_state = Arc::new(AppState { pool });
    let router = Router::new()
        .merge(hypermedia_router::hypermedia_router())
        .merge(data_router::data_router())
        .nest_service("/static", ServeDir::new("./css"))
        .with_state(shared_state);

    Ok(router.into())
}

#[derive(Template)]
#[template(path = "expenses.html")]
struct ExpensesTemplate {
    months: MonthsIter,
    current_month: Months,
}

impl Default for ExpensesTemplate {
    fn default() -> Self {
        Self {
            months: Months::iter(),
            current_month: Months::from_chrono_month(
                Month::try_from(u8::try_from(Utc::now().month()).unwrap()).unwrap(),
            ),
        }
    }
}
