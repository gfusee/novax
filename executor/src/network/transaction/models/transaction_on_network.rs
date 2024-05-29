use serde::Deserialize;

use crate::utils::transaction::results::find_sc_error;

#[derive(Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOnNetworkResponse {
    pub data: Option<TransactionOnNetwork>,
    pub error: String
}

#[derive(Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOnNetwork {
    pub transaction: TransactionOnNetworkTransaction
}

#[derive(Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOnNetworkTransaction {
    pub gas_used: u64,
    pub smart_contract_results: Option<Vec<TransactionOnNetworkTransactionSmartContractResult>>,
    pub status: String,
    pub logs: TransactionOnNetworkTransactionLogs
}

#[derive(Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOnNetworkTransactionSmartContractResult {
    pub hash: String,
    pub nonce: u64,
    pub data: String,
}

#[derive(Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOnNetworkTransactionLogs {
    pub address: String,
    pub events: Vec<TransactionOnNetworkTransactionLogsEvents>
}

#[derive(Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOnNetworkTransactionLogsEvents {
    pub address: String,
    pub identifier: String,
    pub topics: Vec<String>,
    pub data: Option<String>
}

impl TransactionOnNetwork {
    pub fn is_success(&self) -> bool {
        if let Ok(None) = find_sc_error(&self.transaction.logs) {
            true
        } else {
            false
        }
    }
}