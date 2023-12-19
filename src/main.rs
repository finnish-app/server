use std::{fmt::Display, str::FromStr, sync::Arc};

use askama::Template;
use axum::{
    extract::{Path, State},
    //    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
    //Json,
};
use serde::{Deserialize, Serialize};
use shuttle_runtime::CustomError;
use sqlx::{FromRow, PgPool};
use strum::EnumIter;
use tower_http::services::ServeDir;

struct AppState {
    pool: PgPool,
}

#[derive(Serialize, Clone, EnumIter, Deserialize, sqlx::Type)]
#[sqlx(type_name = "expense_type", rename_all = "lowercase")]
enum ExpenseType {
    Food,
    Transport,
    Health,
    Education,
    Entertainment,
    Other,
}

impl FromStr for ExpenseType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Food" => Ok(ExpenseType::Food),
            "Transport" => Ok(ExpenseType::Transport),
            "Health" => Ok(ExpenseType::Health),
            "Education" => Ok(ExpenseType::Education),
            "Entertainment" => Ok(ExpenseType::Entertainment),
            "Other" => Ok(ExpenseType::Other),
            _ => Err(format!("{} is not a valid expense type", s)),
        }
    }
}

impl Display for ExpenseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpenseType::Food => write!(f, "Food"),
            ExpenseType::Transport => write!(f, "Transport"),
            ExpenseType::Health => write!(f, "Health"),
            ExpenseType::Education => write!(f, "Education"),
            ExpenseType::Entertainment => write!(f, "Entertainment"),
            ExpenseType::Other => write!(f, "Other"),
        }
    }
}

#[derive(FromRow, Serialize)]
struct Expense {
    id: i32,
    description: String,
    price: f32,
    expense_type: ExpenseType,
    is_essencial: bool,
}

#[shuttle_runtime::main]
async fn axum(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(CustomError::new)?;

    let shared_state = Arc::new(AppState { pool });
    let router = Router::new()
        .route("/", get(expenses_index))
        .route("/expenses", get(get_expenses))
        .route("/expenses/edit", get(edit_expenses))
        .route("/expenses/:id", get(get_expense).put(update_expense))
        //.route("/whatever", get(whatever))
        .nest_service("/static", ServeDir::new("./css"))
        .with_state(shared_state);

    Ok(router.into())
}

//async fn whatever(
//    State(shared_state): State<Arc<AppState>>,
//) -> Result<impl IntoResponse, impl IntoResponse> {
//    match sqlx::query_as!(Expense, "SELECT * FROM expenses")
//        .fetch_all(&shared_state.pool)
//        .await
//    {
//        Ok(expenses) => Ok((StatusCode::CREATED, Json(expenses))),
//        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
//    }
//}

#[derive(Template)]
#[template(path = "expenses.html")]
struct ExpensesTemplate {}

async fn expenses_index() -> impl IntoResponse {
    ExpensesTemplate {}
}

macro_rules! TABLE_ROW {
    () => {
        "<tr>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>
                <button class=\"btn btn-danger\"
                hx-get=\"/expenses/{}/edit\"
                hx-trigger=\"edit\"
                _=\"on click
                   if .editing is not empty
                       Swal.fire({{title: 'Already Editing',
                                  showCancelButton: true,
                                  confirmButtonText: 'Yep, Edit This Row!',
                                  text:'Hey!  You are already editing a row!  Do you want to cancel that edit and continue?'}})
                       if the result's isConfirmed is false
                         halt
                       end
                       send cancel to .editing
                     end
                     trigger edit\">
                    Edit
                </button>
            </td>
        </tr>"
    };
}

macro_rules! EDITABLE_TABLE_ROW {
    () => {
        "<tr hx-trigger='cancel' class='editing' hx-get=\"/expenses/{}\">
            <td><input type='text' name='description' value='{}'></td>
            <td><input type='number' name='price' value='{}'></td>
            <td><select name='expense_type'>
                <option value='Food'>Food</option>
                <option value='Transport'>Transport</option>
                <option value='Health'>Health</option>
                <option value='Education'>Education</option>
                <option value='Entertainment'>Entertainment</option>
                <option value='Other'>Other</option>
            </select></td>
            <td><input type='checkbox' name='is_essencial' {}></td>
            <td>
                <button class=\"btn btn-danger\" hx-get=\"/expenses/{}\">
                  Cancel
                </button>
                <button class=\"btn btn-danger\" hx-put=\"/expenses/{}\" hx-include=\"closest tr\">
                  Save
                </button>
            </td>
        </tr>"
    };
}

async fn get_expenses(State(shared_state): State<Arc<AppState>>) -> impl IntoResponse {
    //    let expenses: Vec<Expense> = sqlx::query_as!(Expense, "SELECT * FROM expenses")
    //        .fetch_all(&shared_state.pool)
    //        .await
    //        .unwrap();
    let expenses: Vec<Expense> = sqlx::query_as("SELECT * FROM expenses")
        .fetch_all(&shared_state.pool)
        .await
        .unwrap();

    Html(
        expenses
            .iter()
            .map(|expense| {
                format!(
                    TABLE_ROW!(),
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

async fn edit_expenses(State(shared_state): State<Arc<AppState>>) -> impl IntoResponse {
    let expenses: Vec<Expense> = sqlx::query_as("SELECT * FROM expenses")
        .fetch_all(&shared_state.pool)
        .await
        .unwrap();

    Html(
        expenses
            .iter()
            .map(|expense| {
                format!(
                    EDITABLE_TABLE_ROW!(),
                    expense.id,
                    expense.description,
                    expense.price,
                    expense.is_essencial,
                    expense.id,
                    expense.id
                )
            })
            .collect::<Vec<String>>()
            .join("\n"),
    )
}

async fn get_expense(
    Path(id): Path<i32>,
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let expense: Expense = sqlx::query_as("SELECT * FROM expenses WHERE id = $1")
        .bind(id)
        .fetch_one(&shared_state.pool)
        .await
        .unwrap();

    Html(format!(
        TABLE_ROW!(),
        expense.description, expense.price, expense.expense_type, expense.is_essencial, expense.id
    ))
}

async fn update_expense(
    Path(id): Path<i32>,
    State(shared_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let expense: Expense = sqlx::query_as("SELECT * FROM expenses WHERE id = $1")
        .bind(id)
        .fetch_one(&shared_state.pool)
        .await
        .unwrap();

    Html(format!(
        TABLE_ROW!(),
        expense.description, expense.price, expense.expense_type, expense.is_essencial, expense.id
    ))
}
