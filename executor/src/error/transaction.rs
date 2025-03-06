use serde::{Deserialize, Serialize};

use crate::{ExecutorError, TransactionOnNetwork};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum TransactionError {
    EgldAndEsdtPaymentsDetected,
    CannotDeserializeTransactionSendingResponse { response: String },
    ErrorWhileSendingTheTransaction,
    ErrorWhileGettingTransactionOnNetwork { tx_hash: String },
    CannotDeserializeTransactionOnNetworkResponse { response: String },
    FailedToSendTheTransaction { message: String },
    NoSmartContractResult,
    SmartContractExecutionError { status: u64, message: String },
    TimeoutWhenRetrievingTransactionOnNetwork,
    CannotDecodeSmartContractResult { response: TransactionOnNetwork },
    NoSCDeployLogInTheResponse,
    CannotEncodeString { string: String },
    CannotEncodeU64 { value: u64 },
    CannotEncodeTransfer,
    CannotSerializeTransactionData,
    CannotDecodeBase64,
    CannotDecodeTopic,
    WrongTopicsCountForSignalErrorEvent
}

impl From<TransactionError> for ExecutorError {
    fn from(value: TransactionError) -> Self {
        ExecutorError::Transaction(value)
    }
}