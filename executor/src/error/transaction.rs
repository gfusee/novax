use serde::{Deserialize, Serialize};

use crate::ExecutorError;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum TransactionError {
    EgldAndEsdtPaymentsDetected
}

impl From<TransactionError> for ExecutorError {
    fn from(value: TransactionError) -> Self {
        ExecutorError::Transaction(value)
    }
}