use crate::{
    constant::{EDITABLE_TABLE_ROW, TABLE_ROW},
    schema::{Expense, ExpenseType, GetExpense, UpdateExpense},
    util::{get_first_day_from_month_or_none, get_last_day_from_month_or_none},
    AppState, ExpensesTemplate,
};
use std::sync::Arc;

use askama_axum::IntoResponse;
use axum::{
    extract::{Path, Query, State},
    response::Html,
    routing::get,
    Json, Router,
};

pub fn hypermedia_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(expenses_index))
        .route("/expenses", get(get_expenses))
        .route("/expenses/:id/edit", get(edit_expense))
        .route("/expenses/:id", get(get_expense).put(update_expense))
}

pub async fn expenses_index() -> impl IntoResponse {
    ExpensesTemplate {
        ..Default::default()
    }
}

pub async fn get_expenses(
    State(shared_state): State<Arc<AppState>>,
    Query(get_expense_input): Query<GetExpense>,
) -> impl IntoResponse {
    //let expenses: Vec<Expense> = match get_expense_input.month.clone() {
    //    Some(month) => {
    //        sqlx::query_as!(
    //            Expense,
    //            r#"SELECT id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
    //            FROM expenses WHERE date BETWEEN $1 AND $2 ORDER BY date ASC"#,
    //            get_first_day_from_month(month.clone() as u32 + 1),
    //            get_last_day_from_month(month as u32 + 1)
    //        )
    //        .fetch_all(&shared_state.pool)
    //        .await
    //        .unwrap()
    //    }
    //    None => {
    //        sqlx::query_as!(
    //            Expense,
    //            r#"SELECT id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
    //            FROM expenses ORDER BY date ASC"#
    //        )
    //            .fetch_all(&shared_state.pool)
    //            .await
    //            .unwrap()
    //    }
    //};

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
    .fetch_all(&shared_state.pool)
    .await
    .unwrap();

    tracing::info!("expenses: {:?}", expenses);

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
    )
}

pub async fn edit_expense(
    Path(id): Path<i32>,
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let expense = sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
        FROM expenses WHERE id = $1"#,
        id
    )
        .fetch_one(&shared_state.pool)
        .await
        .unwrap();

    Html(format!(
        EDITABLE_TABLE_ROW!(),
        expense.id,
        expense.date,
        expense.description,
        expense.price,
        expense.is_essencial,
        expense.id,
        expense.id
    ))
}

pub async fn get_expense(
    Path(id): Path<i32>,
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let expense = sqlx::query_as!(
        Expense,
        r#"SELECT id, description, price, expense_type as "expense_type: ExpenseType", is_essencial, date
        FROM expenses WHERE id = $1"#,
        id
    )
    .fetch_one(&shared_state.pool)
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
    Path(id): Path<i32>,
    State(shared_state): State<Arc<AppState>>,
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
    .fetch_one(&shared_state.pool)
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
