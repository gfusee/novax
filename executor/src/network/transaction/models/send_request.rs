use serde::Serialize;

#[derive(Serialize)]
pub struct TransactionSendRequest {
    pub nonce: u64,
    pub value: String,
    pub receiver: String,
    pub gas_price: u64,
    pub gas_limit: u64,
    pub data: String,
    pub chain_id: String,
    pub version: u64,
    pub signature: String
}