use askama_axum::IntoResponse;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Response,
    routing::get,
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    schema::{CreateExpense, GetExpense, UpdateExpenseApi},
    AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/expenses", get(list).post(create))
        .route("/api/expenses/:uuid", get(find).put(update).delete(delete))
}

async fn list(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Query(get_expenses_input): Query<GetExpense>,
) -> Result<Json<Vec<crate::queries::expenses::Expense>>, Response> {
    let user_id = if let Some(user) = auth_session.user {
        tracing::info!("User logged in");
        user.id
    } else {
        tracing::error!("User not logged in");
        return Err(StatusCode::UNAUTHORIZED.into_response());
    };

    match crate::features::expenses::list_in_period(
        user_id,
        shared_state.pool.clone(),
        get_expenses_input.from,
        get_expenses_input.to,
    )
    .await
    {
        Ok(expenses) => Ok(Json(expenses)),
        Err(e) => {
            tracing::error!(user_id, "Error inserting expense: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())
        }
    }
}

async fn create(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Json(create_expense): Json<CreateExpense>,
) -> Result<StatusCode, Response> {
    let user_id = if let Some(user) = auth_session.user {
        tracing::info!("User logged in");
        user.id
    } else {
        tracing::error!("User not logged in");
        return Err(StatusCode::UNAUTHORIZED.into_response());
    };

    match crate::features::expenses::create(user_id, shared_state.pool.clone(), create_expense)
        .await
    {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => {
            tracing::error!(user_id, "Error inserting expense: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())
        }
    }
}

async fn find(
    auth_session: AuthSession,
    Path(uuid): Path<Uuid>,
    State(shared_state): State<Arc<AppState>>,
) -> Result<Json<crate::queries::expenses::Expense>, Response> {
    let user_id = if let Some(user) = auth_session.user {
        tracing::info!("User logged in");
        user.id
    } else {
        tracing::error!("User not logged in");
        return Err(StatusCode::UNAUTHORIZED.into_response());
    };

    match crate::features::expenses::find_active_for_user(user_id, shared_state.pool.clone(), uuid)
        .await
    {
        Ok(expense) => Ok(Json(expense)),
        Err(e) => {
            tracing::error!(user_id, "Error inserting expense: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())
        }
    }
}

async fn update(
    auth_session: AuthSession,
    Path(uuid): Path<Uuid>,
    State(shared_state): State<Arc<AppState>>,
    Json(update_expense): Json<UpdateExpenseApi>,
) -> Result<StatusCode, Response> {
    let user_id = if let Some(user) = auth_session.user {
        tracing::info!("User logged in");
        user.id
    } else {
        tracing::error!("User not logged in");
        return Err(StatusCode::UNAUTHORIZED.into_response());
    };

    match crate::features::expenses::update(
        user_id,
        shared_state.pool.clone(),
        uuid,
        update_expense,
    )
    .await
    {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => {
            tracing::error!(user_id, "Error inserting expense: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())
        }
    }
}

async fn delete(
    auth_session: AuthSession,
    Path(uuid): Path<Uuid>,
    State(shared_state): State<Arc<AppState>>,
) -> Result<StatusCode, Response> {
    let user_id = if let Some(user) = auth_session.user {
        tracing::info!("User logged in");
        user.id
    } else {
        tracing::error!("User not logged in");
        return Err(StatusCode::UNAUTHORIZED.into_response());
    };

    let outcome = crate::features::expenses::delete(user_id, shared_state.pool.clone(), uuid)
        .await
        .map_err(|e| {
            tracing::error!(user_id, "Error inserting expense: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        })?;

    match outcome {
        crate::features::expenses::DeleteOutcome::Success => Ok(StatusCode::OK),
        crate::features::expenses::DeleteOutcome::NotFound => Ok(StatusCode::NOT_FOUND),
    }
}
