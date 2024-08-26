use std::sync::Arc;

use crate::client::pluggy::transactions::{ListTransactionsOutcome, ListTransactionsResponse};
use axum::{
    extract::{Query, State},
    http::{header::HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
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
        chrono::Utc::now(),
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

async fn list_accounts(
    auth_session: AuthSession,
    headers: HeaderMap,
    Query(list_accounts_request): Query<ListAccountsRequest>,
) -> Result<Json<ListAccountsResponse>, StatusCode> {
    let Some(user) = auth_session.user else {
        panic!("user not logged in")
    };
    tracing::debug!("User logged in");

    let Some(x_api_key) = headers.get("x-api-key") else {
        tracing::debug!(?user.id, "missing api key header");
        return Err(StatusCode::BAD_REQUEST);
    };
    let Ok(api_key) = x_api_key.to_str() else {
        tracing::debug!(?user.id, "api_key header malformed, couldn't parse chars");
        return Err(StatusCode::BAD_REQUEST);
    };

    match crate::client::pluggy::account::list_accounts(api_key, &list_accounts_request.item_id)
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
    headers: HeaderMap,
    Query(list_transactions_request): Query<ListTransactionsRequest>,
) -> Result<Json<ListTransactionsResponse>, StatusCode> {
    let Some(user) = auth_session.user else {
        panic!("user not logged in")
    };
    tracing::debug!("User logged in");

    let Some(x_api_key) = headers.get("x-api-key") else {
        tracing::debug!(?user.id, "missing api key header");
        return Err(StatusCode::BAD_REQUEST);
    };
    let Ok(api_key) = x_api_key.to_str() else {
        tracing::debug!(?user.id, "api_key header malformed, couldn't parse chars");
        return Err(StatusCode::BAD_REQUEST);
    };

    match crate::client::pluggy::transactions::list_transactions(
        api_key,
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
