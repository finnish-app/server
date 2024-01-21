#![warn(
    clippy::all,
    nonstandard_style,
    future_incompatible,
    missing_debug_implementations
)]
#![forbid(unsafe_code)]

mod auth;
mod client;
mod constant;
mod data;
mod data_structs;
mod hypermedia;
mod schema;
mod util;

use crate::{
    auth::Backend,
    data_structs::{Months, MonthsIter},
};
use std::{sync::Arc, time::Duration};

use askama_axum::Template;
use axum::{error_handling::HandleErrorLayer, http::StatusCode, Router};
use axum_helmet::{
    ContentSecurityPolicy, CrossOriginOpenerPolicy, CrossOriginResourcePolicy, Helmet, HelmetLayer,
    OriginAgentCluster, ReferrerPolicy, StrictTransportSecurity, XContentTypeOptions,
    XDNSPrefetchControl, XDownloadOptions, XFrameOptions, XPermittedCrossDomainPolicies,
    XXSSProtection,
};
use axum_login::{
    login_required,
    tower_sessions::{Expiry, PostgresStore, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use chrono::{Datelike, Month, Utc};
use schema::{ExpenseType, ExpenseTypeIter};
use shuttle_runtime::CustomError;
use shuttle_secrets::SecretStore;
use sqlx::PgPool;
use strum::IntoEnumIterator;
use tower::{BoxError, ServiceBuilder};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct AppState {
    pool: PgPool,
    secret_store: SecretStore,
}

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
    #[shuttle_shared_db::Postgres] pool: PgPool,
) -> shuttle_axum::ShuttleAxum {
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

    // Session layer.
    //
    // This uses `tower-sessions` to establish a layer that will provide the session
    // as a request extension.
    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await.map_err(CustomError::new)?;

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(time::Duration::minutes(30)));

    let backend = Backend::new(pool.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let content_sec_policy = ContentSecurityPolicy::new()
        .default_src(vec!["'self'"])
        .base_uri(vec!["'self'"])
        .font_src(vec!["'self'", "https:", "data:"])
        .form_action(vec!["'self'"])
        .frame_ancestors(vec!["'self'"])
        .img_src(vec!["'self'", "data:"])
        .object_src(vec!["'none'"])
        .script_src(vec!["'self'"])
        .script_src_attr(vec!["'self'"])
        .script_src_elem(vec!["'self'", "https:", "'unsafe-inline'"]) // currently breaks without unsafe-inline in htmx.min
        // this is somehow related to
        // plotting with plotly, but
        // rest of app works normally
        .style_src(vec!["'self'", "https:", "'unsafe-inline'"])
        .upgrade_insecure_requests();

    let helmet_layer = HelmetLayer::new(
        Helmet::new()
            .add(content_sec_policy)
            .add(CrossOriginOpenerPolicy::same_origin())
            .add(CrossOriginResourcePolicy::same_origin())
            .add(OriginAgentCluster::new(true))
            .add(ReferrerPolicy::no_referrer())
            .add(
                StrictTransportSecurity::new()
                    .max_age(15552000)
                    .include_sub_domains(),
            )
            .add(XContentTypeOptions::nosniff())
            .add(XDNSPrefetchControl::off())
            .add(XDownloadOptions::noopen())
            .add(XFrameOptions::Deny)
            .add(XPermittedCrossDomainPolicies::none())
            .add(XXSSProtection::off()),
    );

    let shared_state = Arc::new(AppState { pool, secret_store });
    let router = Router::new()
        .merge(data::router::data_router())
        .merge(hypermedia::router::expenses::router())
        .merge(hypermedia::router::auth::private_router())
        .route_layer(login_required!(Backend, login_url = "/auth"))
        .merge(hypermedia::router::auth::public_router())
        .merge(hypermedia::router::validation::router())
        .nest_service("/static", ServeDir::new("./css"))
        .nest_service("/js", ServeDir::new("./js"))
        .nest_service("/img", ServeDir::new("./img"))
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
        .layer(helmet_layer)
        .layer(auth_layer)
        .with_state(shared_state);

    tracing::debug!("Server started");
    Ok(router.into())
}

#[derive(Template)]
#[template(path = "expenses.html")]
struct ExpensesTemplate<'a> {
    current_month: Months,
    expense_types: ExpenseTypeIter,
    months: MonthsIter,
    username: &'a str,
}

#[derive(Template, Default)]
#[template(path = "change_password.html")]
struct ChangePasswordTemplate {
    change_password_url: String,
    passwords_match_url: String,
    password_strength_url: String,
}

impl Default for ExpensesTemplate<'_> {
    fn default() -> Self {
        Self {
            current_month: Months::from_chrono_month(
                Month::try_from(u8::try_from(Utc::now().month()).unwrap()).unwrap(),
            ),
            expense_types: ExpenseType::iter(),
            months: Months::iter(),
            username: "user",
        }
    }
}

#[derive(Template, Default)]
#[template(path = "auth.html")]
struct AuthTemplate {
    should_print_message_in_signin: u8,
}

#[derive(Template, Default)]
#[template(path = "verify.html")]
struct VerificationTemplate {
    login_url: String,
    message: String,
    resend_url: String,
    should_print_resend_link: bool,
}
