use std::{sync::Arc, time::Duration};

use anyhow::bail;
use sqlx::PgPool;

use crate::client::classifier::TransactionToCategorize;

pub fn renew_pluggy_task(
    pluggy_api_key: Arc<tokio::sync::Mutex<String>>,
    pluggy_client_id: String,
    pluggy_client_secret: String,
) -> tokio::task::JoinHandle<anyhow::Result<()>> {
    tokio::task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60 * 5)).await;
            tracing::debug!("waky waky renew pluggy");

            let crate::client::pluggy::auth::CreateApiKeyOutcome::Success(new_pluggy_api_key) =
                crate::client::pluggy::auth::create_api_key(
                    &pluggy_client_id,
                    &pluggy_client_secret,
                )
                .await?
            else {
                bail!("task couldn't renew pluggy api_key")
            };

            let mut pluggy_api_key = pluggy_api_key.lock().await;
            *pluggy_api_key = new_pluggy_api_key.api_key;
            tracing::info!("renewed pluggy api key");
        }
    })
}

pub fn categorize_transactions_task(
    db_pool: PgPool,
) -> tokio::task::JoinHandle<anyhow::Result<()>> {
    tokio::task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60 * 5)).await;
            tracing::debug!("starting transaction categorization task");

            let uncategorized = match crate::queries::expenses::list_uncategorized(&db_pool).await {
                Ok(rows) => rows,
                Err(e) => {
                    tracing::error!(?e, "failed to fetch uncategorized transactions");
                    continue;
                }
            };

            if uncategorized.is_empty() {
                tracing::debug!("no uncategorized transactions found");
                continue;
            }

            tracing::info!(
                count = uncategorized.len(),
                "processing uncategorized transactions"
            );

            // Convert to TransactionToCategorize format
            let transactions: Vec<TransactionToCategorize> = uncategorized
                .iter()
                .map(|row| TransactionToCategorize {
                    id: row.id.to_string(),
                    description: row.description.clone(),
                    amount: row.price,
                    balance: 0.0,
                    // FIX: use external category
                    category: row.category.clone().unwrap_or_default().to_string(),
                })
                .collect();

            match crate::client::classifier::predict(&transactions).await {
                Ok(transactions_to_update) => {
                    for tx in &transactions_to_update {
                        match crate::queries::expenses::update_category(
                            &db_pool,
                            tx.id,
                            tx.category.clone(),
                        )
                        .await
                        {
                            Ok(c) => {
                                if c.rows_affected() > 1 {
                                    tracing::error!(
                                        "i really need a macro that cancels the transaction"
                                    );
                                }
                                tracing::debug!("confidence_level: {}", tx.confidence_level);
                            }
                            Err(e) => {
                                tracing::error!(?e, tx_id = ?tx.id, "failed to update transaction category");
                            }
                        }
                    }
                    tracing::info!(
                        count = transactions_to_update.len(),
                        "updated transaction categories"
                    );
                }
                Err(e) => {
                    tracing::error!(?e, "failed to predict transaction categories");
                }
            }
        }
    })
}
