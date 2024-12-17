use std::str::FromStr;

use crate::schema::ExpenseCategory;
use anyhow::bail;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct TransactionToCategorize {
    pub id: String,
    pub description: String,
    pub amount: f32,
    pub balance: f32,
    pub category: String,
}

#[derive(Deserialize)]
pub struct PredictionResult {
    pub id: String,
    pub category: String,
    pub confidence_level: f32,
}

#[derive(Deserialize)]
pub struct PredictionResponse {
    pub predictions: Vec<PredictionResult>,
}

#[derive(Debug)]
pub struct PredictRustResponse {
    pub id: i32,
    pub category: ExpenseCategory,
    pub confidence_level: f32,
}

pub async fn predict(
    transactions: &[TransactionToCategorize],
) -> anyhow::Result<Vec<PredictRustResponse>> {
    let client = reqwest::Client::new();
    let resp = client
        .post("https://model.fina.center/transactions")
        .json(transactions)
        .send()
        .await?;

    let response: PredictionResponse = match resp.status() {
        StatusCode::OK => resp.json().await?,
        _ => bail!("unknown error: {}", resp.text().await?),
    };

    let mut results = Vec::new();
    for prediction in response.predictions {
        let expense_category = match ExpenseCategory::from_str(&prediction.category) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(?e, "error categorizing transaction in model");
                bail!("error converting category string into enum: {e:?}")
            }
        };
        results.push(PredictRustResponse {
            id: prediction.id.parse::<i32>()?,
            category: expense_category,
            confidence_level: prediction.confidence_level,
        });
    }

    Ok(results)
}
