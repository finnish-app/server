use std::collections::BTreeMap;

use crate::{
    auth::AuthSession,
    constant::TABLE_ROW,
    schema::{Expense, ExpenseCategory, GetExpense, UpdateExpense},
    templates::{DeleteExpenseModal, EditableExpenseRowTemplate, ExpenseRowTemplate},
    util::{get_first_day_from_month_or_none, get_last_day_from_month_or_none},
};

use askama_axum::IntoResponse;
use axum::{http::StatusCode, response::Html};
use chrono::NaiveDate;
use plotly::{Layout, Plot, Scatter};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub async fn get_expenses(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    get_expense_input: GetExpense,
) -> impl IntoResponse {
    let user_id = if let Some(user) = auth_session.user {
        tracing::info!("User logged in");
        user.id
    } else {
        tracing::error!("User not logged in");
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let first_day_of_month = match get_first_day_from_month_or_none(get_expense_input.month.clone())
    {
        Ok(date) => date,
        Err(e) => {
            tracing::error!("Error getting first day of month: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let last_day_of_month = match get_last_day_from_month_or_none(get_expense_input.month) {
        Ok(date) => date,
        Err(e) => {
            tracing::error!("Error getting last day of month: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let expenses = sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, category as "category: ExpenseCategory", is_essential, date, uuid
        FROM expenses
        WHERE ((date >= $1) OR ($1 IS NULL))
        AND ((date <= $2) OR ($2 IS NULL))
        AND user_id = $3
        ORDER BY date ASC"#,
        first_day_of_month,
        last_day_of_month,
        user_id
    )
    .fetch_all(db_pool)
    .await
    .unwrap();

    return (
        StatusCode::OK,
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
                        expense.category,
                        expense.is_essential,
                        expense.uuid
                    )
                })
                .collect::<Vec<String>>()
                .join("\n"),
        ),
    )
        .into_response();
}

pub async fn edit_expense(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    uuid: Uuid,
) -> impl IntoResponse {
    let user_id = if let Some(user) = auth_session.user {
        tracing::info!("User logged in");
        user.id
    } else {
        tracing::error!("User not logged in");
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let expense = sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, category as "category: ExpenseCategory", is_essential, date, uuid
        FROM expenses WHERE uuid = $1 AND user_id = $2"#,
        uuid,
        user_id
    )
        .fetch_one(db_pool)
        .await
        .unwrap();

    return EditableExpenseRowTemplate {
        uuid: expense.uuid,
        date: expense.date,
        description: expense.description,
        price: expense.price,
        is_essential: if expense.is_essential { "checked" } else { "" },
        current_category: expense.category,
        ..Default::default()
    }
    .into_response();
}

pub async fn get_expense(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    uuid: Uuid,
) -> impl IntoResponse {
    let user_id = if let Some(user) = auth_session.user {
        tracing::info!("User logged in");
        user.id
    } else {
        tracing::error!("User not logged in");
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let expense = sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, category as "category: ExpenseCategory", is_essential, date, uuid
        FROM expenses WHERE uuid = $1 AND user_id = $2"#,
        uuid,
        user_id
    )
    .fetch_one(db_pool)
    .await
    .unwrap();

    return ExpenseRowTemplate {
        date: expense.date,
        description: expense.description,
        price: expense.price,
        category: expense.category,
        is_essential: expense.is_essential,
        uuid: expense.uuid,
    }
    .into_response();
}

pub async fn update_expense(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    uuid: Uuid,
    update_expense: UpdateExpense,
) -> impl IntoResponse {
    let user_id = if let Some(user) = auth_session.user {
        tracing::info!("User logged in");
        user.id
    } else {
        tracing::error!("User not logged in");
        return StatusCode::UNAUTHORIZED.into_response();
    };

    match sqlx::query_as!(
        Expense,
        r#"
        UPDATE expenses SET
            description = COALESCE($1, description),
            price = COALESCE($2, price),
            category = COALESCE($3 :: expense_category, category),
            is_essential = COALESCE($4, is_essential),
            date = COALESCE($5, date)
        WHERE uuid = $6 AND user_id = $7
        RETURNING id, description, price, category as "category: ExpenseCategory", is_essential, date, uuid
        "#,
        update_expense.description,
        update_expense.price,
        update_expense.category as Option<ExpenseCategory>,
        update_expense.is_essential,
        update_expense.date,
        uuid,
        user_id
    )
    .fetch_one(db_pool)
    .await {
        Ok(expense) => {
            (
                StatusCode::OK,
                [("HX-Trigger", "refresh-plots")],
                ExpenseRowTemplate {
                    date: expense.date,
                    description: expense.description,
                    price: expense.price,
                    category: expense.category,
                    is_essential: expense.is_essential,
                    uuid: expense.uuid
                }
            ).into_response()
        },
        Err(e) => {
            tracing::error!("Error updating expense: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        },
    }
}

pub async fn delete_expense(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    uuid: Uuid,
) -> impl IntoResponse {
    let user_id = if let Some(user) = auth_session.user {
        tracing::info!("User logged in");
        user.id
    } else {
        tracing::error!("User not logged in");
        return StatusCode::UNAUTHORIZED.into_response();
    };

    match sqlx::query!(
        r#"
        DELETE FROM expenses
        WHERE uuid = $1 AND user_id = $2
        "#,
        uuid,
        user_id
    )
    .execute(db_pool)
    .await
    {
        Ok(_) => (StatusCode::OK, [("HX-Trigger", "refresh-table")]).into_response(),
        Err(e) => {
            tracing::error!("Error deleting expense: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn remove_expense_modal(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    uuid: Uuid,
) -> impl IntoResponse {
    let user_id = if let Some(user) = auth_session.user {
        tracing::info!("User logged in");
        user.id
    } else {
        tracing::error!("User not logged in");
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let expense = sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, category as "category: ExpenseCategory", is_essential, date, uuid
        FROM expenses WHERE uuid = $1 AND user_id = $2"#,
        uuid,
        user_id
    )
        .fetch_one(db_pool)
        .await
        .unwrap();

    DeleteExpenseModal {
        expense_uuid: expense.uuid,
    }
    .into_response()
}

pub async fn insert_expense(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    create_expense: UpdateExpense,
) -> impl IntoResponse {
    let user_id = if let Some(user) = auth_session.user {
        tracing::info!("User logged in");
        user.id
    } else {
        tracing::error!("User not logged in");
        return StatusCode::UNAUTHORIZED.into_response();
    };

    match sqlx::query_as!(
        Expense,
        r#"
        INSERT INTO expenses (description, price, category, is_essential, date, uuid, user_id)
        VALUES ($1, $2, $3 :: expense_category, $4, $5, $6, $7)
        RETURNING id, description, price, category as "category: ExpenseCategory", is_essential, date, uuid
        "#,
        create_expense.description,
        create_expense.price,
        create_expense.category as Option<ExpenseCategory>,
        create_expense.is_essential,
        create_expense.date,
        Uuid::new_v4(),
        user_id
    )
    .fetch_one(db_pool)
    .await {
        Ok(_) => (
            StatusCode::CREATED,
            [("HX-Trigger", "refresh-table")],
            "Created",
        ).into_response(),
        Err(e) => {
            tracing::error!("Error inserting expense: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        },
    }
}

/// Plots the expenses of the user.
/// This plot is a time series of the expenses of the user.
pub async fn plot_expenses(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    get_expense_input: GetExpense,
) -> impl IntoResponse {
    let user_id = if let Some(user) = auth_session.user {
        tracing::info!("User logged in");
        user.id
    } else {
        tracing::error!("User not logged in");
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let first_day_of_month = match get_first_day_from_month_or_none(get_expense_input.month.clone())
    {
        Ok(date) => date,
        Err(e) => {
            tracing::error!("Error getting first day of month: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let last_day_of_month = match get_last_day_from_month_or_none(get_expense_input.month) {
        Ok(date) => date,
        Err(e) => {
            tracing::error!("Error getting last day of month: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let expenses = match sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, category as "category: ExpenseCategory", is_essential, date, uuid
        FROM expenses
        WHERE ((date >= $1) OR ($1 IS NULL))
        AND ((date <= $2) OR ($2 IS NULL))
        AND user_id = $3
        ORDER BY date ASC"#,
        first_day_of_month,
        last_day_of_month,
        user_id
    )
    .fetch_all(db_pool)
    .await
    {
        Ok(expenses) => expenses,
        Err(e) => {
            tracing::error!("Error getting expenses: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let mut expenses_by_date = BTreeMap::<NaiveDate, f32>::new();
    for expense in expenses {
        let price = expenses_by_date.entry(expense.date).or_insert(0.0);
        *price += expense.price;
    }

    let trace = Scatter::new(
        expenses_by_date.keys().copied().collect(),
        expenses_by_date.values().copied().collect(),
    )
    .name("Expenses");

    let mut plot = Plot::new();
    plot.add_trace(trace);

    let layout = Layout::new().title("Expenses");
    plot.set_layout(layout);

    plot.to_inline_html(Some("plot-data")).into_response()
}
