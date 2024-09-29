use chrono::NaiveDate;
use plotly::{Layout, Plot, Scatter};
use sqlx::{Pool, Postgres};
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::schema::{CreateExpense, UpdateExpenseApi, UpdateExpenseHypr};

pub async fn create(
    user_id: i32,
    db_pool: Pool<Postgres>,
    create_expense: CreateExpense,
) -> Result<(), sqlx::Error> {
    let params = crate::queries::expenses::CreateParams {
        description: create_expense.description,
        price: create_expense.price,
        category: create_expense.category,
        is_essential: create_expense.is_essential,
        date: create_expense.date,
    };
    crate::queries::expenses::create(&db_pool, user_id, params)
        .await
        .map(|c| {
            if c.rows_affected() > 1 {
                tracing::error!("i really need a macro that cancels the transaction");
            }
            Ok(())
        })?
}

pub async fn list_in_period(
    user_id: i32,
    db_pool: Pool<Postgres>,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
) -> Result<Vec<crate::queries::expenses::Expense>, sqlx::Error> {
    crate::queries::expenses::list_for_user_in_period(&db_pool, user_id, from, to).await
}

pub async fn find_active_for_user(
    user_id: i32,
    db_pool: Pool<Postgres>,
    expense_uuid: Uuid,
) -> Result<crate::queries::expenses::Expense, sqlx::Error> {
    crate::queries::expenses::find_active_for_user(&db_pool, user_id, expense_uuid).await
}

pub async fn update(
    user_id: i32,
    db_pool: Pool<Postgres>,
    expense_uuid: Uuid,
    update: UpdateExpenseApi,
) -> Result<(), sqlx::Error> {
    let params = crate::queries::expenses::UpdateParams {
        description: update.description,
        price: update.price,
        category: update.category,
        is_essential: update.is_essential,
        date: update.date,
    };
    crate::queries::expenses::update(&db_pool, user_id, expense_uuid, params)
        .await
        .map(|c| {
            if c.rows_affected() > 1 {
                tracing::error!("i really need a macro that cancels the transaction");
            }
            Ok(())
        })?
}

pub enum UpdateOutcome {
    Success(crate::queries::expenses::Expense),
    NotFound,
}

pub async fn update_for_site(
    user_id: i32,
    db_pool: Pool<Postgres>,
    expense_uuid: Uuid,
    update: UpdateExpenseHypr,
) -> anyhow::Result<UpdateOutcome> {
    if let Some(exists) =
        crate::queries::expenses::exists_active(&db_pool, user_id, expense_uuid).await?
    {
        if !exists {
            tracing::warn!("entered via Some false");
            return Ok(UpdateOutcome::NotFound);
        }
    } else {
        tracing::warn!("entered via None");
        return Ok(UpdateOutcome::NotFound);
    }

    let params = crate::queries::expenses::UpdateParams {
        description: update.description,
        price: update.price,
        category: update.category,
        is_essential: update.is_essential,
        date: update.date,
    };
    let expense =
        crate::queries::expenses::update_for_site(&db_pool, user_id, expense_uuid, params).await?;

    Ok(UpdateOutcome::Success(expense))
}

pub enum DeleteOutcome {
    Success,
    NotFound,
}

pub async fn delete(
    user_id: i32,
    db_pool: Pool<Postgres>,
    expense_uuid: Uuid,
) -> anyhow::Result<DeleteOutcome> {
    if let Some(exists) =
        crate::queries::expenses::exists_active(&db_pool, user_id, expense_uuid).await?
    {
        if !exists {
            tracing::warn!("entered via Some false");
            return Ok(DeleteOutcome::NotFound);
        }
    } else {
        tracing::warn!("entered via None");
        return Ok(DeleteOutcome::NotFound);
    }

    crate::queries::expenses::delete(&db_pool, user_id, expense_uuid, chrono::Utc::now())
        .await
        .map(|c| {
            if c.rows_affected() > 1 {
                tracing::error!("i really need a macro that cancels the transaction");
            }
        })?;

    Ok(DeleteOutcome::Success)
}

pub async fn plot(
    user_id: i32,
    db_pool: Pool<Postgres>,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
) -> Result<Plot, sqlx::Error> {
    let expenses =
        crate::queries::expenses::list_for_user_in_period(&db_pool, user_id, from, to).await?;

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

    Ok(plot)
}
