use crate::{
    constant::{EDITABLE_TABLE_ROW, TABLE_ROW},
    schema::{Expense, ExpenseType, GetExpense, Login, UpdateExpense},
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
use plotly::{common::Title, Layout, Plot, Scatter};
use sqlx::{Pool, Postgres};

pub async fn signin(
    _db_pool: &Pool<Postgres>,
    Json(_signin_input): Json<Login>,
) -> impl IntoResponse {
    //    match sqlx::query!(
    //        r#"SELECT id, username, password FROM users WHERE username = $1 AND password = $2"#,
    //        signin_input.username,
    //        signin_input.password
    //    )
    //    .fetch_one(db_pool)
    //    .await
    let result_was: Result<(), sqlx::Error> = Ok(());
    match result_was {
        Ok(_) => (StatusCode::OK, [("HX-Redirect", "/")], "Logged in").into_response(),
        Err(e) => {
            tracing::error!("Error logging in: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn signin_tab() -> impl IntoResponse {
    Html("
        <div class=\"tab-list\" role=\"tablist\">
        <button hx-get=\"/signin\" class=\"selected\" role=\"tab\" aria-selected=\"false\" aria-controls=\"tab-content\">Tab 1</button>
        <button hx-get=\"/signup\" role=\"tab\" aria-selected=\"false\" aria-controls=\"tab-content\">Tab 2</button>
        </div>

        <div id=\"tab-content\" role=\"tabpanel\" class=\"tab-content\">
        <form id=\"signin-form\" hx-post=\"/signin\" hx-ext=\"my-json-enc\" hx-swap=\"outerHTML\" hx-target=\"#signin-form\">
        <input
          type=\"text\"
          name=\"username\"
          placeholder=\"Username\"
          aria-label=\"Login\"
          autocomplete=\"nickname\"
          required
        />
        <input
          type=\"password\"
          name=\"password\"
          placeholder=\"Password\"
          aria-label=\"Password\"
          autocomplete=\"current-password\"
          required
        />
        <button type=\"submit\" class=\"contrast\">Login</button>
        <fieldset>
            <label for=\"remember\">
                <input type=\"checkbox\" role=\"switch\" id=\"remember\" name=\"remember\" />
                Remember me
            </label>
        </fieldset>
        </form>
        </div>
    ")
}

pub async fn signup_tab() -> impl IntoResponse {
    Html("
        <form id=\"signup-form\" hx-post=\"/signup\" hx-ext=\"my-json-enc\" hx-swap=\"outerHTML\" hx-target=\"#signup-form\">
        <input
          type=\"text\"
          name=\"username\"
          placeholder=\"Username\"
          aria-label=\"Login\"
          autocomplete=\"nickname\"
          required
        />
        <input
          type=\"password\"
          name=\"password\"
          placeholder=\"Password\"
          aria-label=\"Password\"
          autocomplete=\"current-password\"
          required
        />
        <button type=\"submit\" class=\"contrast\">Sign up</button>
    </form>
    ")
}

pub async fn get_expenses(
    db_pool: &Pool<Postgres>,
    Query(get_expense_input): Query<GetExpense>,
) -> impl IntoResponse {
    let expenses = sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
        FROM expenses
        WHERE ((date >= $1) OR ($1 IS NULL))
        AND ((date <= $2) OR ($2 IS NULL))
        ORDER BY date ASC"#,
        get_first_day_from_month_or_none(get_expense_input.month.clone()),
        get_last_day_from_month_or_none(get_expense_input.month)
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

pub async fn edit_expense(db_pool: &Pool<Postgres>, Path(id): Path<i32>) -> impl IntoResponse {
    let expense = sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
        FROM expenses WHERE id = $1"#,
        id
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

pub async fn get_expense(db_pool: &Pool<Postgres>, Path(id): Path<i32>) -> impl IntoResponse {
    let expense = sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
        FROM expenses WHERE id = $1"#,
        id
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
    db_pool: &Pool<Postgres>,
    Path(id): Path<i32>,
    Json(update_expense): Json<UpdateExpense>,
) -> impl IntoResponse {
    let expense = sqlx::query_as!(
        Expense,
        r#"
        UPDATE expenses SET
            description = COALESCE($1, description),
            price = COALESCE($2, price),
            expense_type = COALESCE($3 :: expense_type, expense_type),
            is_essencial = COALESCE($4, is_essencial),
            date = COALESCE($5, date)
        WHERE id = $6
        RETURNING id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
        "#,
        update_expense.description,
        update_expense.price,
        update_expense.expense_type as Option<ExpenseType>,
        update_expense.is_essencial,
        update_expense.date,
        id
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

pub async fn insert_expense(
    db_pool: &Pool<Postgres>,
    Json(create_expense): Json<UpdateExpense>,
) -> impl IntoResponse {
    match sqlx::query_as!(
        Expense,
        r#"
        INSERT INTO expenses (description, price, expense_type, is_essencial, date)
        VALUES ($1, $2, $3 :: expense_type, $4, $5)
        RETURNING id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
        "#,
        create_expense.description,
        create_expense.price,
        create_expense.expense_type as Option<ExpenseType>,
        create_expense.is_essencial,
        create_expense.date
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
    db_pool: &Pool<Postgres>,
    Query(get_expense_input): Query<GetExpense>,
) -> impl IntoResponse {
    let expenses = sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
        FROM expenses
        WHERE ((date >= $1) OR ($1 IS NULL))
        AND ((date <= $2) OR ($2 IS NULL))
        ORDER BY date ASC"#,
        get_first_day_from_month_or_none(get_expense_input.month.clone()),
        get_last_day_from_month_or_none(get_expense_input.month)
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
