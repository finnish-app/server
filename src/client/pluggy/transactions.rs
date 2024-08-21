use anyhow::bail;
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::client::pluggy::auth::ApiKey;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListTransactionsResponse {
    total: i16,
    page: i16,
    total_pages: i16,
    /// List of retrieved transactions
    results: Vec<Transaction>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    id: Uuid,
    description: String,
    description_raw: Option<String>,
    currency_code: String,
    amount: f32,
    /// Transaction amount in Account's Currency.
    /// Only present if the transaction is in a different currency
    /// than the account's currency
    amount_in_account_currency: Option<f32>,
    date: DateTime<Utc>,
    /// Category of the transaction (e.g. Restaurants, Education).
    /// See the Transaction Categorization section in our guides.
    category: String,
    /// Id of the transaction category.
    /// Can be used to identify the category in the Categories endpoint
    category_id: String,
    #[serde(rename = "type")]
    #[allow(clippy::struct_field_names)]
    transaction_type: TransactionType,
    /// Balance after the transaction
    balance: f32,
    /// Institution provided code
    provider_code: String,
    status: TransactionStatus,
    account_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    payment_data: Option<PaymentData>,
    acquirer_data: Option<AcquirerData>,
    credit_card_metadata: Option<CreditCardMetadata>,
    merchant: Option<Merchant>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PaymentData {
    payer: PayerOrReceiver,
    receiver: PayerOrReceiver,
    reason: String,
    reference_number: String,
    receiver_reference_id: String,
    payment_method: PaymentMethod,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum DocumentType {
    Cpf,
    Cnpj,
}

#[derive(Debug, Serialize, Deserialize)]
struct DocumentNumber {
    #[serde(rename = "type")]
    document_type: DocumentType,
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PayerOrReceiver {
    document_number: DocumentNumber,
    name: String,
    account_number: String,
    branch_number: String,
    routing_number: String,
    routing_number_ispb: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum PaymentMethod {
    Ted,
    Doc,
    Pix,
    Tev,
}

#[derive(Debug, Serialize, Deserialize)]
struct AcquirerData {
    #[serde(rename = "type")]
    acquirer_type: AcquirerType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum AcquirerType {
    Sale,
    Receivable,
    Anticipation,
    Chargeback,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreditCardMetadata {
    installment_number: i32,
    total_installments: i32,
    total_amount: i32,
    purchase_date: DateTime<Utc>,
    payee_mcc: String,
    card_number: String,
    bill_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Merchant {
    name: String,
    business_name: String,
    cnpj: String,
    cnae: String,
    category: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum TransactionStatus {
    Posted,
    Pending,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum TransactionType {
    Debit,
    Credit,
}

pub enum ListTransactionsOutcome {
    Success(ListTransactionsResponse),
    Missing,
    Internal,
}

pub async fn list_transactions(
    api_key: &ApiKey,
    account_id: &Uuid,
) -> anyhow::Result<ListTransactionsOutcome> {
    let mut headers = HeaderMap::new();
    let api_key_hdr_value = HeaderValue::from_str(&api_key.api_key)?;
    headers.insert(HeaderName::from_static("x-api-key"), api_key_hdr_value);

    let client = reqwest::Client::new();
    let resp = client
        .get("https:://api.pluggy.ai/transactions")
        .query(&[("accountId", account_id)])
        .headers(headers)
        .send()
        .await?;

    let transactions: ListTransactionsResponse = match resp.status() {
        StatusCode::OK => resp.json().await?,
        StatusCode::BAD_REQUEST => return Ok(ListTransactionsOutcome::Missing),
        StatusCode::INTERNAL_SERVER_ERROR => return Ok(ListTransactionsOutcome::Internal),
        _ => bail!("unknown error: {}", resp.text().await?),
    };

    Ok(ListTransactionsOutcome::Success(transactions))
}
