use crate::{
    auth::{AuthSession, LoginCredentials},
    constant::{EDITABLE_TABLE_ROW, SIGN_IN_TAB, SIGN_UP_TAB, TABLE_ROW},
    schema::{Expense, ExpenseType, GetExpense, UpdateExpense},
    util::{get_first_day_from_month_or_none, get_last_day_from_month_or_none},
};

use askama_axum::IntoResponse;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Html,
    Json,
};
use chrono::NaiveDate;
use password_auth::generate_hash;
use plotly::{common::Title, Layout, Plot, Scatter};
use sqlx::{Pool, Postgres};

pub async fn signin(
    Json(signin_input): Json<LoginCredentials>,
    mut auth_session: AuthSession,
) -> impl IntoResponse {
    let user = match auth_session.authenticate(signin_input).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            todo!()
        }
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    (StatusCode::OK, [("HX-Redirect", "/")], "Logged in").into_response()
}

pub async fn signin_tab() -> impl IntoResponse {
    Html(SIGN_IN_TAB!())
}

pub async fn signup_tab() -> impl IntoResponse {
    Html(SIGN_UP_TAB!())
}

pub async fn signup(
    db_pool: &Pool<Postgres>,
    Json(signup_input): Json<LoginCredentials>,
) -> impl IntoResponse {
    let hashed_pass = generate_hash(&signup_input.password);
    match sqlx::query!(
        r#"INSERT INTO users (username, password) VALUES ($1, $2) RETURNING id"#,
        signup_input.username,
        hashed_pass
    )
    .fetch_one(db_pool)
    .await
    {
        Ok(_) => (StatusCode::OK, [("HX-Redirect", "/auth")], "Created user").into_response(),
        Err(e) => {
            tracing::error!("Error signing up: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn get_expenses(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    Query(get_expense_input): Query<GetExpense>,
) -> impl IntoResponse {
    let user_id = auth_session.user.expect("User not logged in").id;
    let expenses = sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
        FROM expenses
        WHERE ((date >= $1) OR ($1 IS NULL))
        AND ((date <= $2) OR ($2 IS NULL))
        AND user_id = $3
        ORDER BY date ASC"#,
        get_first_day_from_month_or_none(get_expense_input.month.clone()),
        get_last_day_from_month_or_none(get_expense_input.month),
        user_id
    )
    .fetch_all(db_pool)
    .await
    .unwrap();

    (
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
                        expense.expense_type,
                        expense.is_essencial,
                        expense.id
                    )
                })
                .collect::<Vec<String>>()
                .join("\n"),
        ),
    )
}

pub async fn edit_expense(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let user_id = auth_session.user.expect("User not logged in").id;
    let expense = sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
        FROM expenses WHERE id = $1 AND user_id = $2"#,
        id,
        user_id
    )
        .fetch_one(db_pool)
        .await
        .unwrap();

    Html(format!(
        EDITABLE_TABLE_ROW!(),
        expense.id,
        expense.date,
        expense.description,
        expense.price,
        if expense.is_essencial { "checked" } else { "" },
        expense.id,
        expense.id
    ))
}

pub async fn get_expense(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let user_id = auth_session.user.expect("User not logged in").id;
    let expense = sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
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
        expense.expense_type,
        expense.is_essencial,
        expense.id
    ))
}

pub async fn update_expense(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    Path(id): Path<i32>,
    Json(update_expense): Json<UpdateExpense>,
) -> impl IntoResponse {
    let user_id = auth_session.user.expect("User not logged in").id;

    match sqlx::query_as!(
        Expense,
        r#"
        UPDATE expenses SET
            description = COALESCE($1, description),
            price = COALESCE($2, price),
            expense_type = COALESCE($3 :: expense_type, expense_type),
            is_essencial = COALESCE($4, is_essencial),
            date = COALESCE($5, date)
        WHERE id = $6 AND user_id = $7
        RETURNING id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
        "#,
        update_expense.description,
        update_expense.price,
        update_expense.expense_type as Option<ExpenseType>,
        update_expense.is_essencial,
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
                    expense.expense_type,
                    expense.is_essencial,
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

pub async fn insert_expense(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    Json(create_expense): Json<UpdateExpense>,
) -> impl IntoResponse {
    let user_id = auth_session.user.expect("User not logged in").id;
    match sqlx::query_as!(
        Expense,
        r#"
        INSERT INTO expenses (description, price, expense_type, is_essencial, date, user_id)
        VALUES ($1, $2, $3 :: expense_type, $4, $5, $6)
        RETURNING id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
        "#,
        create_expense.description,
        create_expense.price,
        create_expense.expense_type as Option<ExpenseType>,
        create_expense.is_essencial,
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

pub async fn expenses_plots(
    auth_session: AuthSession,
    db_pool: &Pool<Postgres>,
    Query(get_expense_input): Query<GetExpense>,
) -> impl IntoResponse {
    let user_id = auth_session.user.expect("User not logged in").id;
    let expenses = sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
        FROM expenses
        WHERE ((date >= $1) OR ($1 IS NULL))
        AND ((date <= $2) OR ($2 IS NULL))
        AND user_id = $3
        ORDER BY date ASC"#,
        get_first_day_from_month_or_none(get_expense_input.month.clone()),
        get_last_day_from_month_or_none(get_expense_input.month),
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
        expenses_by_date.keys().cloned().collect(),
        expenses_by_date.values().cloned().collect(),
    )
    .name("Expenses");

    let mut plot = Plot::new();
    plot.add_trace(trace);

    let layout = Layout::new().title(Title::new("Expenses"));
    plot.set_layout(layout);

    plot.to_inline_html(Some("plot-data"))
}
