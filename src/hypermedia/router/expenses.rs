use crate::{
    auth::AuthSession,
    constant::TABLE_ROW,
    schema::{CreateExpense, ExpenseCategory, GetExpense, UpdateExpenseHypr},
    templates::{
        DeleteExpenseModal, EditableExpenseRowTemplate, ExpenseRowTemplate, ExpensesTemplate,
    },
    AppState,
};
use std::sync::Arc;
use strum::IntoEnumIterator;

use askama_axum::IntoResponse;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, Response},
    routing::get,
    Form, Router,
};
use uuid::Uuid;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(expenses_index))
        .route("/expenses", get(list).post(create))
        .route("/expenses/:uuid/edit", get(editable_expense))
        .route("/expenses/:uuid/delete-modal", get(remove_expense_modal))
        .route("/expenses/:uuid", get(find).put(update).delete(delete))
        .route("/expenses/plots", get(plots))
        .route("/expenses/pluggy-widget", get(pluggy_widget))
}

async fn expenses_index(auth_session: AuthSession) -> impl IntoResponse {
    match auth_session.user {
        Some(user) => ExpensesTemplate {
            username: user.username,
            ..Default::default()
        }
        .into_response_with_nonce(),
        None => StatusCode::UNAUTHORIZED.into_response(),
    }
}

async fn list(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Query(get_expenses_input): Query<GetExpense>,
) -> Result<Response, Response> {
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
        Ok(expenses) => Ok((
            [("HX-Trigger", "plot-data")],
            Html(
                expenses
                    .iter()
                    .map(|expense| {
                        format!(
                            TABLE_ROW!(),
                            expense.date,
                            expense.description,
                            expense.price,
                            // FIX: don't unwrap or default
                            expense.category.clone().unwrap_or_default(),
                            expense.is_essential,
                            expense.uuid
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("\n"),
            ),
        )
            .into_response()),
        Err(e) => {
            tracing::error!(user_id, "Error inserting expense: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())
        }
    }
}

async fn editable_expense(
    auth_session: AuthSession,
    Path(uuid): Path<Uuid>,
    State(shared_state): State<Arc<AppState>>,
) -> Result<EditableExpenseRowTemplate, Response> {
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
        Ok(expense) => Ok(EditableExpenseRowTemplate {
            date: expense.date,
            description: expense.description,
            price: expense.price,
            // FIX: don't unwrap or default
            current_category: expense.category.clone().unwrap_or_default(),
            is_essential: if expense.is_essential { "checked" } else { "" },
            uuid: expense.uuid,
            expense_categories: ExpenseCategory::iter(),
        }),
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
) -> Result<ExpenseRowTemplate, Response> {
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
        Ok(expense) => Ok(ExpenseRowTemplate {
            date: expense.date,
            description: expense.description,
            price: expense.price,
            // FIX: don't unwrap or default
            category: expense.category.clone().unwrap_or_default(),
            is_essential: expense.is_essential,
            uuid: expense.uuid,
        }),
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
    Form(update_expense): Form<UpdateExpenseHypr>,
) -> Result<Response, Response> {
    let user_id = if let Some(user) = auth_session.user {
        tracing::info!("User logged in");
        user.id
    } else {
        tracing::error!("User not logged in");
        return Err(StatusCode::UNAUTHORIZED.into_response());
    };

    let outcome = crate::features::expenses::update_for_site(
        user_id,
        shared_state.pool.clone(),
        uuid,
        update_expense,
    )
    .await
    .map_err(|e| {
        tracing::error!(user_id, "Error inserting expense: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
    })?;

    match outcome {
        crate::features::expenses::UpdateOutcome::Success(updated) => Ok((
            [("HX-Trigger", "refresh-plots")],
            ExpenseRowTemplate {
                date: updated.date,
                description: updated.description,
                price: updated.price,
                // FIX: don't unwrap or default
                category: updated.category.clone().unwrap_or_default(),
                is_essential: updated.is_essential,
                uuid: updated.uuid,
            },
        )
            .into_response()),
        crate::features::expenses::UpdateOutcome::NotFound => {
            Ok(StatusCode::NOT_FOUND.into_response())
        }
    }
}

async fn delete(
    auth_session: AuthSession,
    Path(uuid): Path<Uuid>,
    State(shared_state): State<Arc<AppState>>,
) -> Result<Response, Response> {
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
        crate::features::expenses::DeleteOutcome::Success => {
            Ok((StatusCode::OK, [("HX-Trigger", "refresh-table")]).into_response())
        }
        crate::features::expenses::DeleteOutcome::NotFound => {
            Ok(StatusCode::NOT_FOUND.into_response())
        }
    }
}

async fn remove_expense_modal(
    auth_session: AuthSession,
    Path(uuid): Path<Uuid>,
) -> Result<DeleteExpenseModal, StatusCode> {
    if let Some(user) = auth_session.user {
        tracing::info!("User logged in");
        user.id
    } else {
        tracing::error!("User not logged in");
        return Err(StatusCode::UNAUTHORIZED);
    };

    Ok(DeleteExpenseModal { expense_uuid: uuid })
}

async fn create(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Form(create_expense): Form<CreateExpense>,
) -> Result<impl IntoResponse, Response> {
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
        Ok(()) => Ok([("HX-Trigger", "refresh-table")]),
        Err(e) => {
            tracing::error!(user_id, "Error inserting expense: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())
        }
    }
}

async fn plots(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
    Query(get_expenses): Query<GetExpense>,
) -> Result<String, Response> {
    let user_id = if let Some(user) = auth_session.user {
        tracing::info!("User logged in");
        user.id
    } else {
        tracing::error!("User not logged in");
        return Err(StatusCode::UNAUTHORIZED.into_response());
    };

    match crate::features::expenses::plot(
        user_id,
        shared_state.pool.clone(),
        get_expenses.from,
        get_expenses.to,
    )
    .await
    {
        Ok(plot) => Ok(plot.to_inline_html(Some("plot-data"))),
        Err(e) => {
            tracing::error!(user_id, "Error inserting expense: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())
        }
    }
}

async fn pluggy_widget(
    auth_session: AuthSession,
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    crate::hypermedia::service::pluggy::widget(
        auth_session,
        shared_state.pool.clone(),
        &shared_state.env,
        &shared_state.pluggy_api_key.lock().await,
    )
    .await
}
