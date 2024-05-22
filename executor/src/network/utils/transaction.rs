use novax_request::gateway::client::GatewayClient;
use crate::error::transaction::TransactionError;
use crate::ExecutorError;
use crate::network::transaction::models::send_request::TransactionSendRequest;
use crate::network::transaction::models::send_response::TransactionSendResponse;
use crate::network::transaction::models::transaction_on_network::{TransactionOnNetworkResponse, TransactionOnNetwork};

pub async fn send_transaction<Client: GatewayClient>(client: &Client, transaction_request: &TransactionSendRequest) -> Result<String, ExecutorError> {
    let Ok((_, Some(text))) = client
        .with_appended_url("/transaction/send")
        .post(transaction_request)
        .await else {
        return Err(TransactionError::ErrorWhileSendingTheTransaction.into())
    };

    let sent_transaction_response: TransactionSendResponse = serde_json::from_str(&text)
        .map_err(|error| TransactionError::CannotDeserializeTransactionSendingResponse { response: text })?;

    let Some(sent_transaction_data) = sent_transaction_response.data else {
        return Err(TransactionError::FailedToSendTheTransaction { message: sent_transaction_response.error }.into())
    };

    Ok(sent_transaction_data.tx_hash)
}

pub async fn get_transaction_on_network<Client: GatewayClient>(client: &Client, tx_hash: &str) -> Result<TransactionOnNetwork, ExecutorError> {
    let url_to_append = format!("transaction/{tx_hash}?withResults=true");
    let Ok((_, Some(text))) = client
        .with_appended_url(&url_to_append)
        .get()
        .await else {
        return Err(TransactionError::ErrorWhileGettingTransactionOnNetwork { tx_hash: tx_hash.to_string() }.into())
    };

    let transaction_on_network_response: TransactionOnNetworkResponse = serde_json::from_str(&text)
        .map_err(|error| TransactionError::CannotDeserializeTransactionOnNetworkResponse { response: text })?;

    let Some(transaction_on_network_data) = transaction_on_network_response.data else {
        return Err(TransactionError::FailedToSendTheTransaction { message: transaction_on_network_response.error }.into())
    };

    Ok(transaction_on_network_data)
}