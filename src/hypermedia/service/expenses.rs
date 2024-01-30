use crate::{
    auth::AuthSession,
    constant::{DELETE_EXPENSE_MODAL, EDITABLE_TABLE_ROW, TABLE_ROW},
    schema::{Expense, ExpenseCategory, GetExpense, UpdateExpense},
    util::{get_first_day_from_month_or_none, get_last_day_from_month_or_none},
};

use askama_axum::IntoResponse;
use axum::{http::StatusCode, response::Html};
use chrono::NaiveDate;
use plotly::{common::Title, Layout, Plot, Scatter};
use sqlx::{Pool, Postgres};
use strum::IntoEnumIterator;

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
        r#"SELECT id, description, price, category as "category: ExpenseCategory", is_essential, date
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
                        expense.id
                    )
                })
                .collect::<Vec<String>>()
                .join("\n"),
        ),
    )
        .into_response();
}

fn select_category(category: &ExpenseCategory) -> String {
    let mut options = String::new();
    for category_option in ExpenseCategory::iter() {
        options.push_str(&format!(
            r#"<option value='{}' {}>{}</option>"#,
            category_option,
            if *category == category_option {
                "selected"
            } else {
                ""
            },
            category_option
        ));
    }
    options
}

pub async fn edit_expense(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    id: i32,
) -> Html<String> {
    let user_id = if let Some(user) = auth_session.user {
        tracing::info!("User logged in");
        user.id
    } else {
        tracing::error!("User not logged in");
        return Html(String::new());
    };

    let expense = sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, category as "category: ExpenseCategory", is_essential, date
        FROM expenses WHERE id = $1 AND user_id = $2"#,
        id,
        user_id
    )
        .fetch_one(db_pool)
        .await
        .unwrap();

    Html(format!(
        EDITABLE_TABLE_ROW!(),
        id = expense.id,
        date = expense.date,
        description = expense.description,
        price = expense.price,
        is_essential = if expense.is_essential { "checked" } else { "" },
        category = select_category(&expense.category)
    ))
}

pub async fn get_expense(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    id: i32,
) -> Html<String> {
    let user_id = if let Some(user) = auth_session.user {
        tracing::info!("User logged in");
        user.id
    } else {
        tracing::error!("User not logged in");
        return Html(String::new());
    };

    let expense = sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, category as "category: ExpenseCategory", is_essential, date
        FROM expenses WHERE id = $1 AND user_id = $2"#,
        id,
        user_id
    )
    .fetch_one(db_pool)
    .await
    .unwrap();

    Html(format!(
        TABLE_ROW!(),
        expense.date,
        expense.description,
        expense.price,
        expense.category,
        expense.is_essential,
        expense.id
    ))
}

pub async fn update_expense(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    id: i32,
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
        WHERE id = $6 AND user_id = $7
        RETURNING id, description, price, category as "category: ExpenseCategory", is_essential, date
        "#,
        update_expense.description,
        update_expense.price,
        update_expense.category as Option<ExpenseCategory>,
        update_expense.is_essential,
        update_expense.date,
        id,
        user_id
    )
    .fetch_one(db_pool)
    .await {
        Ok(expense) => {
            (
                StatusCode::OK,
                [("HX-Trigger", "refresh-plots")],
                Html(format!(
                    TABLE_ROW!(),
                    expense.date,
                    expense.description,
                    expense.price,
                    expense.category,
                    expense.is_essential,
                    expense.id
                    )
                ),
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
    id: i32,
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
        WHERE id = $1 AND user_id = $2
        "#,
        id,
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
    id: i32,
) -> Html<String> {
    let user_id = if let Some(user) = auth_session.user {
        tracing::info!("User logged in");
        user.id
    } else {
        tracing::error!("User not logged in");
        return Html(String::new());
    };

    let expense = sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, category as "category: ExpenseCategory", is_essential, date
        FROM expenses WHERE id = $1 AND user_id = $2"#,
        id,
        user_id
    )
        .fetch_one(db_pool)
        .await
        .unwrap();

    Html(format!(DELETE_EXPENSE_MODAL!(), expense.id))
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
        INSERT INTO expenses (description, price, category, is_essential, date, user_id)
        VALUES ($1, $2, $3 :: expense_category, $4, $5, $6)
        RETURNING id, description, price, category as "category: ExpenseCategory", is_essential, date
        "#,
        create_expense.description,
        create_expense.price,
        create_expense.category as Option<ExpenseCategory>,
        create_expense.is_essential,
        create_expense.date,
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
    let expenses = sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, category as "category: ExpenseCategory", is_essential, date
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

    let mut expenses_by_date = std::collections::BTreeMap::<NaiveDate, f32>::new();
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

    let layout = Layout::new().title(Title::new("Expenses"));
    plot.set_layout(layout);

    plot.to_inline_html(Some("plot-data")).into_response()
}
