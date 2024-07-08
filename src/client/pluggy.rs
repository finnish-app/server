use std::collections::HashMap;

use anyhow::bail;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use shuttle_runtime::SecretStore;

#[allow(dead_code)]
pub async fn create_api_key(secret_store: &SecretStore) -> anyhow::Result<ApiKey> {
    let client_id = secret_store.get("PLUGGY_CLIENT_ID").unwrap_or_else(|| {
        tracing::error!("Error getting PLUGGY_CLIENT_ID from secret store");
        String::new()
    });
    let client_secret = secret_store.get("PLUGGY_CLIENT_SECRET").unwrap_or_else(|| {
        tracing::error!("Error getting PLUGGY_CLIENT_SECRET from secret store");
        String::new()
    });

    create_api_key_inner(&client_id, &client_secret).await
}

#[derive(Serialize, Deserialize)]
pub struct ApiKey {
    #[serde(rename = "apiKey")]
    pub api_key: String,
}

#[allow(dead_code)]
async fn create_api_key_inner(client_id: &str, client_secret: &str) -> anyhow::Result<ApiKey> {
    let client = reqwest::Client::new();
    let mut map = HashMap::new();
    map.insert("clientId", client_id);
    map.insert("clientSecret", client_secret);

    let resp = client
        .post("https://api.pluggy.ai/auth")
        .json(&map)
        .send()
        .await?;

    let api_key = match resp.status() {
        StatusCode::OK => resp.json().await?,
        StatusCode::UNAUTHORIZED => bail!("401 - unauthorized"),
        StatusCode::INTERNAL_SERVER_ERROR => bail!("500 - internal server error"),
        _ => bail!("unknown error: {}", resp.text().await?),
    };

    Ok(api_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use shuttle_common::Secret;
    use std::collections::BTreeMap;

    #[tokio::test]
    async fn test_create_api_key() {
        let mut secrets: BTreeMap<String, Secret<String>> = BTreeMap::new();
        secrets.insert(
            "PLUGGY_CLIENT_ID".to_string(),
            Secret::new("<think_of_a_way_to_upload>".to_string()),
        );
        secrets.insert(
            "PLUGGY_CLIENT_SECRET".to_string(),
            Secret::new("<think_of_a_way_to_upload>".to_string()),
        );
        let secret_store = shuttle_runtime::SecretStore::new(secrets);

        let api_key: ApiKey = create_api_key(&secret_store).await.unwrap();
        println!("api_key: {}", api_key.api_key);
        panic!()
    }
}
