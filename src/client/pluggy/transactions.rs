use anyhow::bail;
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTransactionsResponse {
    total: i16,
    page: i16,
    total_pages: i16,
    /// List of retrieved transactions
    results: Vec<Transaction>,
}

#[derive(Deserialize, Serialize)]
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
    date: OffsetDateTime,
    /// Category of the transaction (e.g. Restaurants, Education).
    /// See the Transaction Categorization section in our guides.
    category: String,
    /// Id of the transaction category.
    /// Can be used to identify the category in the Categories endpoint
    category_id: Option<String>,
    #[serde(rename = "type")]
    tx_type: TransactionType,
    /// Balance after the transaction
    balance: f32,
    /// Institution provided code
    provider_code: Option<String>,
    status: TransactionStatus,
    account_id: Uuid,
    created_at: Option<OffsetDateTime>,
    updated_at: Option<OffsetDateTime>,
    payment_data: Option<PaymentData>,
    acquirer_data: Option<AcquirerData>,
    credit_card_metadata: Option<CreditCardMetadata>,
    merchant: Option<Merchant>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PaymentData {
    payer: PayerOrReceiver,
    receiver: PayerOrReceiver,
    reason: Option<String>,
    reference_number: String,
    receiver_reference_id: Option<String>,
    payment_method: PaymentMethod,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum DocumentType {
    Cpf,
    Cnpj,
}

#[derive(Deserialize, Serialize)]
struct DocumentNumber {
    #[serde(rename = "type")]
    document_type: DocumentType,
    value: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PayerOrReceiver {
    document_number: DocumentNumber,
    name: String,
    account_number: String,
    branch_number: String,
    routing_number: Option<String>,
    routing_number_ispb: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum PaymentMethod {
    Ted,
    Doc,
    Pix,
    Tev,
}

#[derive(Deserialize, Serialize)]
struct AcquirerData {
    #[serde(rename = "type")]
    acquirer_type: AcquirerType,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum AcquirerType {
    Sale,
    Receivable,
    Anticipation,
    Chargeback,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreditCardMetadata {
    installment_number: i32,
    total_installments: i32,
    total_amount: i32,
    purchase_date: OffsetDateTime,
    payee_mcc: Option<String>,
    card_number: String,
    bill_id: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Merchant {
    name: String,
    business_name: String,
    cnpj: String,
    cnae: Option<String>,
    category: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum TransactionStatus {
    Posted,
    Pending,
}

#[derive(Deserialize, Serialize)]
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
    api_key: &str,
    account_id: &Uuid,
) -> anyhow::Result<ListTransactionsOutcome> {
    let mut headers = HeaderMap::new();
    let api_key_hdr_value = HeaderValue::from_str(api_key)?;
    headers.insert(HeaderName::from_static("x-api-key"), api_key_hdr_value);

    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.pluggy.ai/transactions")
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

#[cfg(test)]
mod tests {
    use super::ListTransactionsResponse;

    const TEST_DATA: &str = r#"
{
  "total": 7,
  "totalPages": 1,
  "page": 1,
  "results": [
    {
      "id": "a8534c85-53ce-4f21-94d7-50e9d2ee4957",
      "description": "* PROV * COMPRA TESOURO DIRETO CLIENTES",
      "descriptionRaw": "* PROV * COMPRA TESOURO DIRETO CLIENTES",
      "currencyCode": "BRL",
      "amount": -212.45,
      "date": "2020-10-15T00:00:00.000Z",
      "balance": 4439.4,
      "category": "Fixed Income Investment",
      "accountId": "562b795d-1653-429f-be86-74ead9502813",
      "providerCode": null,
      "status": "POSTED",
      "paymentData": null,
      "type": "DEBIT"
    },
    {
      "id": "05c693bf-c196-47ea-a28c-8251d6bb8a06",
      "description": "PAGO NETFLIX SERV",
      "descriptionRaw": "PAGO NETFLIX SERV",
      "currencyCode": "USD",
      "amount": -58,
      "amountInAccountCurrency": -298.19,
      "date": "2020-10-15T00:00:00.000Z",
      "balance": 4651.85,
      "category": "Video streaming",
      "accountId": "562b795d-1653-429f-be86-74ead9502813",
      "providerCode": null,
      "status": "POSTED",
      "paymentData": null,
      "type": "DEBIT",
      "merchant": {
        "name": "Netflix",
        "businessName": "NETFLIX ENTRETENIMENTO BRASIL LTDA.",
        "cnpj": "00000000000000",
        "category": "Video streaming",
        "cnae": "5911100"
      }
    },
    {
      "id": "97536285-cc22-4a5a-9d05-f5fe24410d0c",
      "description": "* PROV * DEVOLUÇÃO DE MARGEM",
      "descriptionRaw": "* PROV * DEVOLUÇÃO DE MARGEM",
      "currencyCode": "BRL",
      "amount": 2482.26,
      "date": "2020-10-15T00:00:00.000Z",
      "balance": 4950.04,
      "category": "Margin Withdrawn",
      "accountId": "562b795d-1653-429f-be86-74ead9502813",
      "providerCode": null,
      "status": "POSTED",
      "paymentData": null,
      "type": "CREDIT"
    },
    {
      "id": "8caf328b-4528-4de6-b931-10639d0084c5",
      "description": "LIQUIDO DAS OPERAÇÕES BMF PR. 14/10/2020 NC. 870947",
      "descriptionRaw": "LIQUIDO DAS OPERAÇÕES BMF PR. 14/10/2020 NC. 870947",
      "currencyCode": "BRL",
      "amount": -1.06,
      "date": "2020-10-14T00:00:00.000Z",
      "balance": 2467.78,
      "category": "Investment",
      "accountId": "562b795d-1653-429f-be86-74ead9502813",
      "providerCode": null,
      "status": "POSTED",
      "paymentData": null,
      "type": "DEBIT"
    },
    {
      "id": "ff9ed929-edc4-408c-a959-d51f79ab1814",
      "description": "MERCADOLIVRE*2PRODUTOS",
      "currencyCode": "BRL",
      "amount": 159.2,
      "date": "2020-10-14T00:00:00.000Z",
      "balance": 2468.84,
      "category": "Investment",
      "accountId": "562b795d-1653-429f-be86-74ead9502813",
      "providerCode": null,
      "status": "POSTED",
      "paymentData": null,
      "type": "CREDIT",
      "creditCardMetadata": {
        "totalAmount": 320,
        "totalInstallments": 2,
        "installmentNumber": 2,
        "purchaseDate": "2020-09-14T00:00:00.000Z",
        "payeeMCC": 1234,
        "cardNumber": "0597",
        "billId": "abced929-edc4-408c-a959-d51f79ab1123"
      }
    },
    {
      "id": "093fc873-442a-4bd8-9171-51f17892fb09",
      "description": "LIQUIDO DAS OPERAÇÕES BM&F PR. 14/10/2020 NC. 870947",
      "descriptionRaw": "LIQUIDO DAS OPERAÇÕES BM&F PR. 14/10/2020 NC. 870947",
      "currencyCode": "BRL",
      "amount": -10.3,
      "date": "2020-10-14T00:00:00.000Z",
      "balance": 2309.64,
      "category": "Investment",
      "accountId": "562b795d-1653-429f-be86-74ead9502813",
      "providerCode": null,
      "status": "POSTED",
      "paymentData": null,
      "type": "DEBIT"
    },
    {
      "id": "6ec156fe-e8ac-4d9a-a4b3-7770529ab01c",
      "description": "TED Example",
      "descriptionRaw": null,
      "currencyCode": "BRL",
      "amount": 1500,
      "date": "2020-10-14T00:00:00.000Z",
      "balance": 3500,
      "category": "Transfer",
      "accountId": "03cc0eff-4ec5-495c-adb3-1ef9611624fc",
      "providerCode": "123456",
      "type": "CREDIT",
      "status": "POSTED",
      "paymentData": {
        "payer": {
          "name": "Tiago Rodrigues Santos",
          "branchNumber": "090",
          "accountNumber": "1234-5",
          "routingNumber": "001",
          "documentNumber": {
            "type": "CPF",
            "value": "882.937.076-23"
          }
        },
        "reason": "Taxa de serviço",
        "receiver": {
          "name": "Pluggy",
          "branchNumber": "999",
          "accountNumber": "9876-1",
          "routingNumber": "002",
          "documentNumber": {
            "type": "CNPJ",
            "value": "08.050.608/0001-32"
          }
        },
        "paymentMethod": "TED",
        "referenceNumber": "123456789"
      }
    }
  ]
}
    "#;

    #[test]
    fn test_deserialize() {
        let transactions: ListTransactionsResponse = serde_json::from_str(TEST_DATA).unwrap();
        assert_eq!(transactions.results.len(), 7);
    }
}
