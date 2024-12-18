use anyhow::Context;
use plotly::{Layout, Plot, Scatter};
use sqlx::PgPool;
use std::{cmp::Reverse, collections::BTreeMap};
use time::{Date, OffsetDateTime};
use uuid::Uuid;

use crate::schema::{CreateExpense, UpdateExpenseApi, UpdateExpenseHypr};

pub async fn create(
    user_id: i32,
    db_pool: PgPool,
    create_expense: CreateExpense,
) -> Result<(), sqlx::Error> {
    let params = crate::queries::expenses::CreateParams {
        description: create_expense.description,
        price: create_expense.price,
        category: create_expense.category,
        is_essential: create_expense.is_essential,
        date: create_expense.date,
        now: OffsetDateTime::now_utc(),
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

pub async fn process_pluggy_expenses(
    user_id: i32,
    db_pool: PgPool,
    pluggy_api_key: &str,
) -> anyhow::Result<()> {
    let item_ids =
        match crate::queries::pluggy_items::find_latests_for_user(&db_pool, user_id).await {
            Ok(ids) => ids,
            Err(e) => {
                tracing::error!(?e, "database error on find latest user item_id");
                Vec::new()
            }
        };

    for item in item_ids {
        let accounts =
            crate::client::pluggy::account::list_accounts(pluggy_api_key, &item.external_item_id)
                .await
                .context("failed to list accounts from pluggy api")?
                .results;

        for account in accounts {
            let most_recent =
                crate::queries::expenses::most_recent_for_account(&db_pool, user_id, account.id)
                    .await?;

            let crate::client::pluggy::transactions::ListTransactionsOutcome::Success(res) =
                crate::client::pluggy::transactions::list_transactions(
                    pluggy_api_key,
                    &account.id,
                    most_recent
                        .as_ref()
                        .map(|t| t.date - time::Duration::days(1))
                        .as_ref(),
                )
                .await
                .context("failed to list transactions from pluggy api")?
            else {
                tracing::error!("don't know yet");
                panic!("don't know yet")
            };

            let transactions = res.results;
            tracing::info!("current transactions: {transactions:#?}");

            for transaction in transactions {
                if let Some(recent_timestamps) = most_recent.as_ref() {
                    if transaction.created_at <= recent_timestamps.external_created_at {
                        tracing::debug!(
                            ?user_id,
                            ?transaction.id,
                            ?transaction.created_at,
                            ?recent_timestamps.external_created_at,
                            "skipped inserting already inserted transaction"
                        );
                        continue;
                    }
                }

                let params = crate::queries::expenses::CreateParamsFromPluggy {
                    description: transaction.description,
                    price: transaction.amount,
                    category: None,
                    bank: Some(item.connector_name.clone()),
                    external_account_id: transaction.account_id,
                    external_id: transaction.id,
                    external_created_at: transaction.created_at,
                    is_essential: false,
                    date: transaction.date.date(),
                    now: OffsetDateTime::now_utc(),
                };
                let res =
                    crate::queries::expenses::insert_from_pluggy(&db_pool, user_id, params).await?;
                if res.rows_affected() > 1 {
                    tracing::error!("i really need a macro that cancels the transaction");
                }
            }
        }
    }

    Ok(())
}

pub async fn list_in_period(
    user_id: i32,
    db_pool: PgPool,
    from: Option<Date>,
    to: Option<Date>,
) -> Result<Vec<crate::queries::expenses::Expense>, sqlx::Error> {
    let mut expenses =
        crate::queries::expenses::list_for_user_in_period(&db_pool, user_id, from, to).await?;

    expenses.sort_unstable_by_key(|e| Reverse(e.date));

    Ok(expenses)
}

pub async fn find_active_for_user(
    user_id: i32,
    db_pool: PgPool,
    expense_uuid: Uuid,
) -> Result<crate::queries::expenses::Expense, sqlx::Error> {
    crate::queries::expenses::find_active_for_user(&db_pool, user_id, expense_uuid).await
}

pub async fn update(
    user_id: i32,
    db_pool: PgPool,
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
    db_pool: PgPool,
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
    db_pool: PgPool,
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

    crate::queries::expenses::delete(&db_pool, user_id, expense_uuid, OffsetDateTime::now_utc())
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
    db_pool: PgPool,
    from: Option<Date>,
    to: Option<Date>,
) -> Result<Plot, sqlx::Error> {
    let expenses =
        crate::queries::expenses::list_for_user_in_period(&db_pool, user_id, from, to).await?;

    let mut expenses_by_date = BTreeMap::<Date, f32>::new();
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

#[cfg(test)]
mod tests {
    use sqlx::PgPool;
    use time::OffsetDateTime;

    #[sqlx::test]
    async fn create_success(pool: PgPool) {
        let email = "test@test.com";
        let now = OffsetDateTime::now_utc();
        crate::queries::user::create(
            &pool,
            crate::queries::user::CreateParams {
                name: "test_name",
                email,
                hashed_pass: "test_hash",
                verification_token: "test_token",
                expiration_date: now,
            },
        )
        .await
        .unwrap();

        let user_id = sqlx::query!("SELECT id FROM users WHERE email = $1", email)
            .fetch_one(&pool)
            .await
            .unwrap()
            .id;

        let create_expense = crate::schema::CreateExpense {
            description: "test_expense".to_owned(),
            price: 6.9,
            category: crate::schema::ExpenseCategory::Restaurants,
            is_essential: Some(false),
            date: now.date(),
        };

        super::create(user_id, pool, create_expense).await.unwrap();
    }
}
