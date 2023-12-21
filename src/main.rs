mod data;
mod data_router;
mod hypermedia_router;
mod schema;
mod util;

use crate::data::{Months, MonthsIter};
use std::{sync::Arc, time::Duration};

use askama::Template;
use axum::{error_handling::HandleErrorLayer, http::StatusCode, Router};
use chrono::{Datelike, Month, Utc};
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
        .merge(hypermedia_router::hypermedia_router())
        .merge(data_router::data_router())
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
