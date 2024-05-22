use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSendResponse {
    pub data: Option<TransactionSendResponseData>,
    pub error: String
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSendResponseData {
    pub tx_hash: String
}