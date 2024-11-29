mod auth;
mod client;
mod constant;
mod data;
mod features;
mod hypermedia;
mod queries;
/// Module containing the database schemas and i/o schemas for hypermedia and data apis.
mod schema;
/// Module containing the askama html templates to be rendered.
mod templates;
/// Module containing time and crypto utility functions.
mod util;

use crate::auth::Backend;
use std::{net::SocketAddr, sync::Arc, time::Duration};

use anyhow::{bail, Context};
use axum::{
    body::Body,
    error_handling::HandleErrorLayer,
    http::{HeaderValue, Response, StatusCode},
    Router,
};
use axum_helmet::{
    ContentSecurityPolicy, CrossOriginOpenerPolicy, CrossOriginResourcePolicy, Helmet, HelmetLayer,
    OriginAgentCluster, ReferrerPolicy, StrictTransportSecurity, XContentTypeOptions,
    XDNSPrefetchControl, XDownloadOptions, XFrameOptions, XPermittedCrossDomainPolicies,
    XXSSProtection,
};
use axum_login::{
    permission_required,
    tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower::{timeout::error::Elapsed, BoxError, ServiceBuilder};
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tower_sessions_sqlx_store::PostgresStore;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct Env {
    db_url: String,
    frc_apikey: String,
    frc_sitekey: String,
    instance_id: String,
    mail_from: String,
    otel_endpoint: String,
    pluggy_client_id: String,
    pluggy_client_secret: String,
    rust_log: String,
    smtp_host: String,
    smtp_key: String,
    smtp_username: String,
    svix_api_key: String,
}

impl Env {
    fn try_build() -> Result<Self, std::env::VarError> {
        // opens
        let db_url = std::env::var("DATABASE_URL")?;

        // secrets
        let frc_apikey = std::env::var("FRC_APIKEY")?;
        let frc_sitekey = std::env::var("FRC_SITEKEY")?;
        let smtp_key = std::env::var("SMTP_KEY")?;
        let smtp_host = std::env::var("SMTP_HOST")?;
        let mail_from = std::env::var("MAIL_FROM")?;
        let smtp_username = std::env::var("SMTP_USERNAME")?;
        let pluggy_client_id = std::env::var("PLUGGY_CLIENT_ID")?;
        let pluggy_client_secret = std::env::var("PLUGGY_CLIENT_SECRET")?;
        let svix_api_key = std::env::var("SVIX_API_KEY")?;

        let instance_id = std::env::var("INSTANCE_ID")?;
        let otel_endpoint = std::env::var("OTEL_ENDPOINT")?;
        let rust_log = std::env::var("RUST_LOG")?;

        Ok(Self {
            db_url,
            frc_apikey,
            frc_sitekey,
            instance_id,
            mail_from,
            otel_endpoint,
            pluggy_client_id,
            pluggy_client_secret,
            rust_log,
            smtp_host,
            smtp_key,
            smtp_username,
            svix_api_key,
        })
    }
}

/// The application state to be shared in axum.
struct AppState {
    /// The postgres connection pool.
    pool: PgPool,
    /// The environment variables
    env: Env,
    /// The pluggy api key
    pluggy_api_key: Arc<tokio::sync::Mutex<String>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = Env::try_build().context("couldn't build envs")?;

    let logger_provider = init_otlp(
        &env.rust_log,
        &env.otel_endpoint,
        "server",
        "0",
        &env.instance_id,
    )
    .context("can't start logging")?;

    let db_pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&env.db_url)
        .await
        .context("can't connect to database")?;

    sqlx::migrate!()
        .run(&db_pool)
        .await
        .context("couldn't run migrations")?;

    let session_store = PostgresStore::new(db_pool.clone());
    session_store
        .migrate()
        .await
        .context("couldn't migrate session store")?;

    let deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );

    let client::pluggy::auth::CreateApiKeyOutcome::Success(pluggy_api_key) =
        client::pluggy::auth::create_api_key(&env.pluggy_client_id, &env.pluggy_client_secret)
            .await?
    else {
        bail!("Couldn't create pluggy api key")
    };
    let pluggy_api_key = Arc::new(tokio::sync::Mutex::new(pluggy_api_key.api_key));

    let shared_state = Arc::new(AppState {
        pool: db_pool,
        env,
        pluggy_api_key,
    });

    let renew_pluggy = renew_pluggy_task(
        shared_state.pluggy_api_key.clone(),
        shared_state.env.pluggy_client_id.clone(),
        shared_state.env.pluggy_client_secret.clone(),
    );

    tracing::info!("Server started");
    let rest = rest(shared_state.clone(), session_store);
    rest.await??;

    tracing::info!("Starting deletion task"); // message won't show
    deletion_task.await??;

    renew_pluggy.await??;

    logger_provider.shutdown()?;
    Ok(())
}

fn rest(
    app_state: Arc<AppState>,
    session_store: PostgresStore,
) -> tokio::task::JoinHandle<anyhow::Result<()>> {
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(time::Duration::minutes(30)));

    let backend = Backend::new(app_state.pool.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let gov_conf = Arc::new(
        GovernorConfigBuilder::default()
            .finish()
            .unwrap_or_default(),
    );
    let governor_limiter = gov_conf.limiter().clone();
    tokio::task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            tracing::info!("rate limiting storage size: {}", governor_limiter.len());
            governor_limiter.retain_recent();
        }
    });

    let helmet_layer = HelmetLayer::new(generate_general_helmet_headers());

    let router = Router::new()
        .merge(data::pluggy::router())
        .merge(data::expenses::router())
        .merge(hypermedia::router::expenses::router())
        .route_layer(permission_required!(
            Backend,
            login_url = "/auth/mfa",
            "restricted:read",
        ))
        .merge(data::auth::private_router())
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
                .layer(GovernorLayer { config: gov_conf })
                .layer(auth_layer)
                .layer(helmet_layer)
                .map_response(|mut res: Response<Body>| {
                    if res.headers().get("content-security-policy").is_none() {
                        res.headers_mut().insert(
                            "content-security-policy",
                            generate_default_csp()
                                .to_string()
                                .parse()
                                .unwrap_or_else(|_| {
                                    tracing::error!("Failed to parse default CSP");
                                    return HeaderValue::from_static(fallback_static_str_csp());
                                }),
                        );
                    }
                    return res;
                })
                .into_inner(),
        )
        .with_state(app_state);

    // TODO: add graceful shutdown: https://github.com/maxcountryman/axum-login/blob/9c26b37cd03be8d803ae261b7bc556229c2043da/examples/sqlite/src/web/app.rs#L69
    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
            .await
            .context("could not start listener")?;
        let server = axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        );

        tracing::info!("REST ready to go at http://127.0.0.1:8000");
        let outcome = server.await;
        tracing::info!("REST went bye bye.");
        outcome.context("server")
    })
}

fn renew_pluggy_task(
    pluggy_api_key: Arc<tokio::sync::Mutex<String>>,
    pluggy_client_id: String,
    pluggy_client_secret: String,
) -> tokio::task::JoinHandle<anyhow::Result<()>> {
    tokio::task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60 * 5)).await;
            tracing::info!("waky waky renew pluggy");

            let client::pluggy::auth::CreateApiKeyOutcome::Success(new_pluggy_api_key) =
                client::pluggy::auth::create_api_key(&pluggy_client_id, &pluggy_client_secret)
                    .await?
            else {
                bail!("task couldn't renew pluggy api_key")
            };

            let mut pluggy_api_key = pluggy_api_key.lock().await;
            *pluggy_api_key = new_pluggy_api_key.api_key;
        }
    })
}

/// Returns a default configuration of http security headers.
fn generate_general_helmet_headers() -> Helmet {
    return Helmet::new()
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
        .add(XXSSProtection::off());
}

/// Returns a default strict Content Security Policy.
/// It's used whenever a custom CSP is not set.
fn generate_default_csp() -> ContentSecurityPolicy<'static> {
    return ContentSecurityPolicy::new()
        .default_src(vec!["'self'"])
        .base_uri(vec!["'none'"])
        .font_src(vec!["'none'"])
        .form_action(vec!["'none'"])
        .frame_src(vec!["'none'"])
        .frame_ancestors(vec!["'none'"])
        .img_src(vec!["'none'"])
        .object_src(vec!["'none'"])
        .script_src(vec!["'none'"])
        .style_src(vec!["'none'"])
        .worker_src(vec!["'none'"])
        .upgrade_insecure_requests();
}

/// Returns a default strict Content Security Policy as a static string.
const fn fallback_static_str_csp() -> &'static str {
    return "
    default-src 'self';
    base-uri 'none';
    font-src 'none';
    form-action 'none';
    frame-src 'none';
    frame-ancestors 'none';
    img-src 'none';
    object-src 'none';
    script-src 'none';
    style-src 'none';
    worker-src 'none';
    upgrade-insecure-requests;
    ";
}

fn init_otlp(
    rust_log: &str,
    otel_endpoint: &str,
    name: &'static str,
    version: &'static str,
    instance_id: &str,
) -> Result<opentelemetry_sdk::logs::LoggerProvider, opentelemetry_sdk::logs::LogError> {
    let logger_provider = init_logs(otel_endpoint, name, version, instance_id)?;

    let otel_logger =
        opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&logger_provider);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(rust_log))
        .with(tracing_subscriber::fmt::layer())
        .with(otel_logger)
        .init();

    Ok(logger_provider)
}

fn init_logs(
    otel_endpoint: &str,
    name: &'static str,
    version: &'static str,
    instance_id: &str,
) -> Result<opentelemetry_sdk::logs::LoggerProvider, opentelemetry_sdk::logs::LogError> {
    use opentelemetry_otlp::{WithExportConfig, WithTonicConfig};
    use opentelemetry_semantic_conventions::resource::{
        SERVICE_INSTANCE_ID, SERVICE_NAME, SERVICE_VERSION,
    };

    let exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(otel_endpoint)
        .with_compression(opentelemetry_otlp::Compression::Zstd)
        .build()?;

    Ok(opentelemetry_sdk::logs::LoggerProvider::builder()
        .with_resource(opentelemetry_sdk::Resource::new([
            opentelemetry::KeyValue::new(SERVICE_NAME, name),
            opentelemetry::KeyValue::new(SERVICE_VERSION, version),
            opentelemetry::KeyValue::new(SERVICE_INSTANCE_ID, instance_id.to_owned()),
        ]))
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .build())
}
