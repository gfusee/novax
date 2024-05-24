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
    CannotDecodeSmartContractResult,
    NoSCDeployLogInTheResponse,
    CannotEncodeString { string: String },
    CannotEncodeU64 { value: u64 },
    CannotEncodeTransfer,
    CannotSerializeTransactionData
}

impl From<TransactionError> for ExecutorError {
    fn from(value: TransactionError) -> Self {
        ExecutorError::Transaction(value)
    }
}