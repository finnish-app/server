mod constant;
mod data;
mod data_structs;
mod hypermedia;
mod schema;
mod util;

use crate::data_structs::{Months, MonthsIter};
use std::{sync::Arc, time::Duration};

use askama_axum::Template;
use axum::{error_handling::HandleErrorLayer, http::StatusCode, Router};
use chrono::{Datelike, Month, Utc};
use schema::{ExpenseType, ExpenseTypeIter};
use shuttle_runtime::CustomError;
use sqlx::PgPool;
use strum::IntoEnumIterator;
use tower::{BoxError, ServiceBuilder};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct AppState {
    pool: PgPool,
}

#[shuttle_runtime::main]
async fn axum(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "finnish=debug,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(CustomError::new)?;

    let shared_state = Arc::new(AppState { pool });
    let router = Router::new()
        .merge(hypermedia::router::hypermedia_router())
        .merge(data::router::data_router())
        .nest_service("/static", ServeDir::new("./css"))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {error}"),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .with_state(shared_state);

    tracing::debug!("Server started");
    Ok(router.into())
}

#[derive(Template)]
#[template(path = "expenses.html")]
struct ExpensesTemplate {
    expense_types: ExpenseTypeIter,
    months: MonthsIter,
    current_month: Months,
}

impl Default for ExpensesTemplate {
    fn default() -> Self {
        Self {
            expense_types: ExpenseType::iter(),
            months: Months::iter(),
            current_month: Months::from_chrono_month(
                Month::try_from(u8::try_from(Utc::now().month()).unwrap()).unwrap(),
            ),
        }
    }
}

#[derive(Template, Default)]
#[template(path = "signin.html")]
struct SignInTemplate {}
