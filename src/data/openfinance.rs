use std::sync::Arc;

use crate::client::pluggy::transactions::{ListTransactionsOutcome, ListTransactionsResponse};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{auth::AuthSession, client::pluggy::account::ListAccountsResponse, AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/pluggyconnect/token", get(connect_token))
        .route("/api/pluggyconnect/success", post(connect_success))
        .route("/api/accounts", get(list_accounts))
        .route("/api/transactions", get(list_transactions))
        .route("/api/refresh-transactions", get(refresh_transactions))
}

#[derive(Deserialize)]
struct ConnectTokenRequest {
    maybe_item_id: Option<Uuid>,
}

#[derive(Serialize)]
struct ConnectTokenResponse {
    connect_token: String,
}

async fn connect_token(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Query(pluggyconnect_request): Query<ConnectTokenRequest>,
) -> Result<Json<ConnectTokenResponse>, StatusCode> {
    use crate::client::pluggy::auth::CreateConnectTokenOutcome;

    let Some(user) = auth_session.user else {
        panic!("user not logged in")
    };

    match crate::client::pluggy::auth::create_connect_token(
        &shared_state.pluggy_api_key.lock().await,
        "https://webhook.fina.center".to_owned(),
        user.id,
        pluggyconnect_request.maybe_item_id,
    )
    .await
    {
        Ok(CreateConnectTokenOutcome::Success(token)) => Ok(Json(ConnectTokenResponse {
            connect_token: token.access_token,
        })),
        Ok(CreateConnectTokenOutcome::NotFound | CreateConnectTokenOutcome::Internal) => {
            tracing::error!(user.id, "couldn't create connect token in pluggy");
            Err(StatusCode::FAILED_DEPENDENCY)
        }
        Ok(CreateConnectTokenOutcome::Forbidden) => {
            tracing::error!(user.id, "error authenticating with api key");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(e) => {
            tracing::error!(?e, user.id, "error creating connect token");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
struct ConnectRequest {
    item_id: Uuid,
    connector_name: String,
}

async fn connect_success(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Json(pluggyconnect_request): Json<ConnectRequest>,
) -> StatusCode {
    let Some(user) = auth_session.user else {
        panic!("user not logged in")
    };

    let maybe_item_id = match crate::queries::pluggy_items::find_item(
        &shared_state.pool,
        user.id,
        pluggyconnect_request.item_id,
    )
    .await
    {
        Ok(res) => res.map(|i| i.id),
        Err(e) => {
            tracing::error!(user.id, ?e, "error finding item_id for user");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    if let Some(item_id) = maybe_item_id {
        match crate::queries::pluggy_items::updated(
            &shared_state.pool,
            user.id,
            item_id,
            OffsetDateTime::now_utc(),
        )
        .await
        {
            Ok(c) => {
                if c.rows_affected() > 1 {
                    tracing::error!("i really need a macro that cancels the transaction");
                }
                StatusCode::OK
            }
            Err(e) => {
                tracing::error!(?e, user.id, "error updating pluggy connect item");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    } else {
        match crate::features::openfinance::add_item(
            shared_state.pool.clone(),
            user.id,
            pluggyconnect_request.item_id,
            pluggyconnect_request.connector_name,
            OffsetDateTime::now_utc(),
        )
        .await
        {
            Ok(()) => StatusCode::CREATED,
            Err(e) => {
                tracing::error!(?e, user.id, "error receiving pluggy connect item_id");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

#[derive(Deserialize)]
struct ListAccountsRequest {
    item_id: Uuid,
}

async fn list_accounts(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Query(list_accounts_request): Query<ListAccountsRequest>,
) -> Result<Json<ListAccountsResponse>, StatusCode> {
    let Some(user) = auth_session.user else {
        panic!("user not logged in")
    };

    match crate::client::pluggy::account::list_accounts(
        &shared_state.pluggy_api_key.lock().await,
        &list_accounts_request.item_id,
    )
    .await
    {
        Ok(accounts) => Ok(Json(accounts)),
        Err(e) => {
            tracing::error!(?e, ?user.id, "could not list accounts");
            Err(StatusCode::FAILED_DEPENDENCY)
        }
    }
}

#[derive(Deserialize, Serialize)]
struct ListTransactionsRequest {
    account_id: Uuid,
}

async fn list_transactions(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Query(list_transactions_request): Query<ListTransactionsRequest>,
) -> Result<Json<ListTransactionsResponse>, StatusCode> {
    let Some(user) = auth_session.user else {
        panic!("user not logged in")
    };

    match crate::client::pluggy::transactions::list_transactions(
        &shared_state.pluggy_api_key.lock().await,
        &list_transactions_request.account_id,
        None,
    )
    .await
    {
        Ok(ListTransactionsOutcome::Success(transactions)) => Ok(Json(transactions)),
        Ok(ListTransactionsOutcome::Missing) => Err(StatusCode::BAD_REQUEST),
        Ok(ListTransactionsOutcome::Internal) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Err(e) => {
            tracing::error!(?e, ?user.id, "could not list transactions");
            Err(StatusCode::FAILED_DEPENDENCY)
        }
    }
}

async fn refresh_transactions(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
) -> StatusCode {
    let Some(user) = auth_session.user else {
        panic!("user not logged in")
    };

    tracing::info!("got in to refresh");

    if let Err(e) = crate::features::expenses::process_pluggy_expenses(
        user.id,
        shared_state.pool.clone(),
        &shared_state.pluggy_api_key.lock().await,
    )
    .await
    {
        tracing::error!(?e, ?user.id, "could not refresh transactions");
        return StatusCode::FAILED_DEPENDENCY;
    }

    StatusCode::OK
}
