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

