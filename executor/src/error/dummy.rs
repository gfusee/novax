use serde::{Deserialize, Serialize};

use crate::ExecutorError;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum DummyExecutorError {
    NoTransactionSent
}

impl From<DummyExecutorError> for ExecutorError {
    fn from(value: DummyExecutorError) -> Self {
        ExecutorError::Dummy(value)
    }
}