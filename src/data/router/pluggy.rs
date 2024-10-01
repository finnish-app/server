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
        .route("/api/pluggyconnect/success", post(connect_success))
        .route("/api/accounts", get(list_accounts))
        .route("/api/transactions", get(list_transactions))
}

#[derive(Deserialize, Serialize)]
struct ConnectRequest {
    item_id: Uuid,
}

// TODO: create and move to service
async fn connect_success(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Json(pluggyconnect_request): Json<ConnectRequest>,
) -> StatusCode {
    let Some(user) = auth_session.user else {
        panic!("user not logged in")
    };
    tracing::debug!("User logged in");

    match crate::features::openfinance::add_item(
        &shared_state.pool,
        user.id,
        pluggyconnect_request.item_id,
        OffsetDateTime::now_utc(),
    )
    .await
    {
        Ok(r) => {
            assert!(r.rows_affected() < 2);
            StatusCode::CREATED
        }

        Err(e) => {
            tracing::error!(?e, user.id, "error receiving pluggy connect item_id");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[derive(Deserialize)]
struct ListAccountsRequest {
    item_id: Uuid,
}

// TODO: create and move to service
async fn list_accounts(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Query(list_accounts_request): Query<ListAccountsRequest>,
) -> Result<Json<ListAccountsResponse>, StatusCode> {
    let Some(user) = auth_session.user else {
        panic!("user not logged in")
    };
    tracing::debug!("User logged in");

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

// TODO: create and move to service
async fn list_transactions(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Query(list_transactions_request): Query<ListTransactionsRequest>,
) -> Result<Json<ListTransactionsResponse>, StatusCode> {
    let Some(user) = auth_session.user else {
        panic!("user not logged in")
    };
    tracing::debug!("User logged in");

    match crate::client::pluggy::transactions::list_transactions(
        &shared_state.pluggy_api_key.lock().await,
        &list_transactions_request.account_id,
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
