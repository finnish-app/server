use std::collections::HashMap;

use anyhow::bail;
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ApiKey {
    #[serde(rename = "apiKey")]
    pub api_key: String,
}

pub enum CreateApiKeyOutcome {
    Success(ApiKey),
    Unauthorized,
    Internal,
}

pub async fn create_api_key(
    client_id: &str,
    client_secret: &str,
) -> anyhow::Result<CreateApiKeyOutcome> {
    let mut map = HashMap::new();
    map.insert("clientId", client_id);
    map.insert("clientSecret", client_secret);

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.pluggy.ai/auth")
        .json(&map)
        .send()
        .await?;

    let api_key: ApiKey = match resp.status() {
        StatusCode::OK => resp.json().await?,
        StatusCode::UNAUTHORIZED => return Ok(CreateApiKeyOutcome::Unauthorized),
        StatusCode::INTERNAL_SERVER_ERROR => return Ok(CreateApiKeyOutcome::Internal),
        _ => bail!("unknown error: {}", resp.text().await?),
    };

    Ok(CreateApiKeyOutcome::Success(api_key))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateConnectTokenParams {
    item_id: Option<String>,
    options: CreateConnectTokenOptions,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateConnectTokenOptions {
    client_user_id: String,
    webhook_url: String,
    avoid_duplicates: bool,
}

#[derive(Deserialize)]
pub struct ConnectToken {
    #[serde(rename = "accessToken")]
    pub access_token: String,
}

pub enum CreateConnectTokenOutcome {
    Success(ConnectToken),
    Forbidden,
    NotFound,
    Internal,
}

pub async fn create_connect_token(
    api_key: &str,
    webhook_url: String,
    user_id: i32,
    update_item_id: Option<uuid::Uuid>,
) -> anyhow::Result<CreateConnectTokenOutcome> {
    let mut headers = HeaderMap::new();
    let api_key_hdr_value = HeaderValue::from_str(api_key)?;
    headers.insert(HeaderName::from_static("x-api-key"), api_key_hdr_value);

    let params = CreateConnectTokenParams {
        item_id: update_item_id.map(|i| i.to_string()),
        options: CreateConnectTokenOptions {
            client_user_id: user_id.to_string(),
            avoid_duplicates: true,
            webhook_url,
        },
    };

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.pluggy.ai/connect_token")
        .headers(headers)
        .json(&params)
        .send()
        .await?;

    let connect_token: ConnectToken = match resp.status() {
        StatusCode::OK => resp.json().await?,
        StatusCode::FORBIDDEN => return Ok(CreateConnectTokenOutcome::Forbidden),
        StatusCode::NOT_FOUND => return Ok(CreateConnectTokenOutcome::NotFound),
        StatusCode::INTERNAL_SERVER_ERROR => return Ok(CreateConnectTokenOutcome::Internal),
        _ => bail!("unknown error: {}", resp.text().await?),
    };

    Ok(CreateConnectTokenOutcome::Success(connect_token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_api_key_success() {
        let pluggy_client_id = std::env::var("PLUGGY_CLIENT_ID").unwrap();
        let pluggy_client_secret = std::env::var("PLUGGY_CLIENT_SECRET").unwrap();

        let outcome = create_api_key(&pluggy_client_id, &pluggy_client_secret)
            .await
            .unwrap();
        assert!(matches!(outcome, CreateApiKeyOutcome::Success(_)));
    }

    #[tokio::test]
    async fn test_create_api_key_unauthorized() {
        let pluggy_client_id = "82b871ed-ff67-4732-8b1a-87f9eb167a0e".to_owned();
        let pluggy_client_secret = "82b871ed-ff67-4732-8b1a-87f9eb167a0e".to_owned();

        let outcome = create_api_key(&pluggy_client_id, &pluggy_client_secret)
            .await
            .unwrap();
        assert!(matches!(outcome, CreateApiKeyOutcome::Unauthorized));
    }

    #[tokio::test]
    async fn test_create_api_key_bad_request() {
        let pluggy_client_id = "unproper uuid".to_owned();
        let pluggy_client_secret = "unproper uuid".to_owned();

        let outcome = create_api_key(&pluggy_client_id, &pluggy_client_secret).await;
        assert!(outcome.is_err());
    }

    #[tokio::test]
    async fn test_create_connect_token_success() {
        let pluggy_client_id = std::env::var("PLUGGY_CLIENT_ID").unwrap();
        let pluggy_client_secret = std::env::var("PLUGGY_CLIENT_SECRET").unwrap();

        let CreateApiKeyOutcome::Success(api_key) =
            create_api_key(&pluggy_client_id, &pluggy_client_secret)
                .await
                .unwrap()
        else {
            unreachable!()
        };

        let outcome = create_connect_token(
            &api_key.api_key,
            "https://testwebhook.com".to_owned(),
            1,
            None,
        )
        .await
        .unwrap();
        assert!(matches!(outcome, CreateConnectTokenOutcome::Success(_)));
    }

    #[tokio::test]
    async fn test_create_connect_token_forbidden() {
        let api_key = ApiKey {
            api_key: "eyJhbGciOiJIUzI1NiJ9.eyJSb2xlIjoiQWRtaW4iLCJJc3N1ZXIiOiJJc3N1ZXIiLCJVc2VybmFtZSI6IkphdmFJblVzZSIsImV4cCI6MTcyMDQ4MzM5MCwiaWF0IjoxNzIwNDgzMzkwfQ.g_hft2ZzWndL2RFVXOGSx6yym-NApxB8QX0DX7WYs38".to_owned()
        };

        let outcome = create_connect_token(
            &api_key.api_key,
            "https://testwebhook.com".to_owned(),
            1,
            None,
        )
        .await
        .unwrap();
        assert!(matches!(outcome, CreateConnectTokenOutcome::Forbidden));
    }

    #[tokio::test]
    async fn test_create_connect_token_empty() {
        let api_key = ApiKey {
            api_key: String::new(),
        };

        let outcome = create_connect_token(
            &api_key.api_key,
            "https://testwebhook.com".to_owned(),
            1,
            None,
        )
        .await
        .unwrap();
        assert!(matches!(outcome, CreateConnectTokenOutcome::Forbidden));
    }
}
