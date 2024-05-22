use serde::{Deserialize, Serialize};

use crate::ExecutorError;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum TransactionError {
    EgldAndEsdtPaymentsDetected,
    CannotDeserializeTransactionSendingResponse { response: String },
    ErrorWhileSendingTheTransaction,
    ErrorWhileGettingTransactionOnNetwork { tx_hash: String },
    CannotDeserializeTransactionOnNetworkResponse { response: String },
    FailedToSendTheTransaction { message: String },
    NoSmartContractResult,
    CannotDecodeSmartContractResult
}

impl From<TransactionError> for ExecutorError {
    fn from(value: TransactionError) -> Self {
        ExecutorError::Transaction(value)
    }
}