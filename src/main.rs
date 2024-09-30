mod auth;
mod client;
mod constant;
mod data;
mod data_structs;
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

use anyhow::bail;
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
    tower_sessions::{session_store, ExpiredDeletion, Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use shuttle_runtime::{tokio::net::TcpListener, CustomError, SecretStore};
use sqlx::PgPool;
use tokio::task;
use tower::{timeout::error::Elapsed, BoxError, ServiceBuilder};
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tower_sessions_sqlx_store::PostgresStore;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

struct Secrets {
    frc_sitekey: String,
    frc_apikey: String,
    smtp_key: String,
    smtp_host: String,
    mail_from: String,
    smtp_username: String,
    pluggy_client_id: String,
    pluggy_client_secret: String,
    svix_api_key: String,
}

impl Secrets {
    fn from_store(secret_store: &SecretStore) -> anyhow::Result<Secrets> {
        let Some(frc_apikey) = secret_store.get("FRC_APIKEY") else {
            bail!("FRC_APIKEY not set in shuttle secret store");
        };

        let Some(frc_sitekey) = secret_store.get("FRC_SITEKEY") else {
            bail!("FRC_SITEKEY not set in shuttle secret store");
        };

        let Some(smtp_key) = secret_store.get("SMTP_KEY") else {
            bail!("SMTP_KEY not set in shuttle secret store");
        };

        let Some(smtp_host) = secret_store.get("SMTP_HOST") else {
            bail!("SMTP_HOST not set in shuttle secret store");
        };

        let Some(mail_from) = secret_store.get("MAIL_FROM") else {
            bail!("MAIL_FROM not set in shuttle secret store");
        };

        let Some(smtp_username) = secret_store.get("SMTP_USERNAME") else {
            bail!("SMTP_USERNAME not set in shuttle secret store");
        };

        let Some(pluggy_client_id) = secret_store.get("PLUGGY_CLIENT_ID") else {
            bail!("PLUGGY_CLIENT_ID not set in shuttle secret store");
        };

        let Some(pluggy_client_secret) = secret_store.get("PLUGGY_CLIENT_SECRET") else {
            bail!("PLUGGY_CLIENT_SECRET not set in shuttle secret store");
        };

        let Some(svix_api_key) = secret_store.get("SVIX_API_KEY") else {
            bail!("SVIX_API_KEY not set in shuttle secret store");
        };

        Ok(Secrets {
            frc_sitekey,
            frc_apikey,
            smtp_key,
            smtp_host,
            mail_from,
            smtp_username,
            pluggy_client_id,
            pluggy_client_secret,
            svix_api_key,
        })
    }
}

/// The application state to be shared in axum.
struct AppState {
    /// The postgres connection pool.
    pool: PgPool,
    /// The shuttle secret store.
    secrets: Secrets,
    /// The pluggy api key
    pluggy_api_key: Arc<tokio::sync::Mutex<String>>,
}

#[shuttle_runtime::main]
#[expect(
    clippy::too_many_lines,
    reason = "I have to think on how to shrink it, idk"
)]
/// The main function of the application.
async fn axum(
    #[shuttle_runtime::Secrets] secret_store: SecretStore,
    #[shuttle_shared_db::Postgres] pool: PgPool,
) -> Result<CustomService, shuttle_runtime::Error> {
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

    let secrets = Secrets::from_store(&secret_store)?;

    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await.map_err(CustomError::new)?;

    let deletion_task = task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );

    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(time::Duration::minutes(30)));

    let backend = Backend::new(pool.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let gov_conf = Arc::new(
        GovernorConfigBuilder::default()
            .finish()
            .unwrap_or_default(),
    );
    let governor_limiter = gov_conf.limiter().clone();
    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            tracing::info!("rate limiting storage size: {}", governor_limiter.len());
            governor_limiter.retain_recent();
        }
    });

    let helmet_layer = HelmetLayer::new(generate_general_helmet_headers());

    let client::pluggy::auth::CreateApiKeyOutcome::Success(pluggy_api_key) =
        client::pluggy::auth::create_api_key(
            &secrets.pluggy_client_id,
            &secrets.pluggy_client_secret,
        )
        .await?
    else {
        return Err(shuttle_runtime::Error::BuildPanic(
            "Couldn't create pluggy api key".to_string(),
        ));
    };
    let pluggy_api_key = Arc::new(tokio::sync::Mutex::new(pluggy_api_key.api_key));

    let shared_state = Arc::new(AppState {
        pool,
        secrets,
        pluggy_api_key,
    });
    let router = Router::new()
        .merge(data::router::pluggy::router())
        .merge(data::router::expenses::router())
        .merge(hypermedia::router::expenses::router())
        .route_layer(permission_required!(
            Backend,
            login_url = "/auth/mfa",
            "restricted:read",
        ))
        .merge(data::router::auth::private_router())
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
        .with_state(shared_state.clone());

    let renew_pluggy = renew_pluggy_task(
        shared_state.pluggy_api_key.clone(),
        shared_state.secrets.pluggy_client_id.clone(),
        shared_state.secrets.pluggy_client_secret.clone(),
    );

    tracing::debug!("Server started");
    Ok(CustomService {
        router,
        deletion_task,
        renew_pluggy,
    })
}

#[derive(Debug)]
/// The custom service to be used in the shuttle runtime.
pub struct CustomService {
    /// The axum router.
    router: Router,
    /// The task to delete expired sessions.
    deletion_task: task::JoinHandle<Result<(), session_store::Error>>,
    /// The task to renew pluggy api keys
    renew_pluggy: task::JoinHandle<Result<(), shuttle_runtime::Error>>,
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for CustomService {
    async fn bind(mut self, addr: SocketAddr) -> Result<(), shuttle_runtime::Error> {
        axum::serve(
            TcpListener::bind(addr).await.map_err(CustomError::new)?,
            self.router
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .map_err(CustomError::new)?;

        let _deletion = tokio::join!(self.deletion_task);
        let _renew_pluggy = tokio::join!(self.renew_pluggy);

        Ok(())
    }
}

fn renew_pluggy_task(
    pluggy_api_key: Arc<tokio::sync::Mutex<String>>,
    pluggy_client_id: String,
    pluggy_client_secret: String,
) -> task::JoinHandle<Result<(), shuttle_runtime::Error>> {
    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60 * 5)).await;

            let client::pluggy::auth::CreateApiKeyOutcome::Success(new_pluggy_api_key) =
                client::pluggy::auth::create_api_key(&pluggy_client_id, &pluggy_client_secret)
                    .await?
            else {
                return Err(shuttle_runtime::Error::Custom(CustomError::msg(
                    "task couldn't renew pluggy api_key",
                )));
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
