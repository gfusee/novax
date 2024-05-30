use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSendRequest {
    pub nonce: u64,
    pub value: String,
    pub receiver: String,
    pub sender: String,
    pub gas_price: u64,
    pub gas_limit: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    pub signature: String,
    #[serde(rename = "chainID")]
    pub chain_id: String,
    pub version: u32,
    #[serde(skip_serializing_if = "is_zero")]
    pub options: u32,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(num: &u32) -> bool {
    *num == 0
}