#![warn(
    clippy::all,
//    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
//    clippy::cargo,
    nonstandard_style,
    future_incompatible,
    missing_debug_implementations
)]
#![allow(
    clippy::single_call_fn,
    clippy::std_instead_of_core,
    clippy::std_instead_of_alloc,
    clippy::needless_return,
    clippy::module_name_repetitions,
    clippy::multiple_unsafe_ops_per_block,
    clippy::question_mark_used,
    clippy::min_ident_chars
)]
#![forbid(unsafe_code)]

mod auth;
mod client;
mod constant;
//mod data;
mod data_structs;
mod hypermedia;
/// Module containing the database schemas and i/o schemas for hypermedia and data apis.
mod schema;
/// Module containing the askama html templates to be rendered.
mod templates;
/// Module containing time and crypto utility functions.
mod util;

use crate::{auth::Backend, data_structs::Months};
use std::{sync::Arc, time::Duration};

use axum::{error_handling::HandleErrorLayer, http::StatusCode, Router};
use axum_helmet::{
    CrossOriginOpenerPolicy, CrossOriginResourcePolicy, Helmet, HelmetLayer, OriginAgentCluster,
    ReferrerPolicy, StrictTransportSecurity, XContentTypeOptions, XDNSPrefetchControl,
    XDownloadOptions, XFrameOptions, XPermittedCrossDomainPolicies, XXSSProtection,
};
use axum_login::{
    permission_required,
    tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use shuttle_runtime::CustomError;
use shuttle_secrets::SecretStore;
use sqlx::PgPool;
use tower::{timeout::error::Elapsed, BoxError, ServiceBuilder};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tower_sessions_sqlx_store::PostgresStore;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// The application state to be shared in axum.
struct AppState {
    /// The postgres connection pool.
    pool: PgPool,
    /// The shuttle secret store.
    secret_store: SecretStore,
}

#[shuttle_runtime::main]
/// The main function of the application.
async fn axum(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
    #[shuttle_shared_db::Postgres] pool: PgPool,
) -> shuttle_axum::ShuttleAxum {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            return "finnish=debug,axum_login=debug,tower_sessions=debug,sqlx=warn,tower_http=debug,axum::rejection=trace".into();
        }))
        .with(fmt::layer())
        .init();

    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(CustomError::new)?;

    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await.map_err(CustomError::new)?;
    let _deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    ); // TODO: create a way to run this task

    let session_layer = SessionManagerLayer::new(session_store)
        //.with_secure(false)
        .with_expiry(Expiry::OnInactivity(time::Duration::minutes(30)));

    let backend = Backend::new(pool.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let helmet_layer = HelmetLayer::new(
        Helmet::new()
            .add(CrossOriginOpenerPolicy::same_origin())
            .add(CrossOriginResourcePolicy::same_origin())
            .add(OriginAgentCluster::new(true))
            .add(ReferrerPolicy::no_referrer())
            .add(
                StrictTransportSecurity::new()
                    .max_age(15_552_000)
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
        //.merge(data::router::data_router())
        .merge(hypermedia::router::expenses::router())
        .route_layer(permission_required!(
            Backend,
            login_url = "/auth/mfa",
            "restricted:read",
        ))
        .merge(hypermedia::router::auth::private_router())
        .route_layer(permission_required!(
            Backend,
            login_url = "/auth/signin",
            "protected:read",
        ))
        .merge(hypermedia::router::auth::public_router())
        .merge(hypermedia::router::validation::router())
        .nest_service("/static", ServeDir::new("./css"))
        .nest_service("/js", ServeDir::new("./js"))
        .nest_service("/img", ServeDir::new("./img"))
        .layer(helmet_layer)
        .layer(auth_layer)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| {
                    return async move {
                        if error.is::<Elapsed>() {
                            return Ok(StatusCode::REQUEST_TIMEOUT);
                        }
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {error}"),
                        ));
                    };
                }))
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .with_state(shared_state);

    tracing::debug!("Server started");
    Ok(router.into())
}
