use serde::{Deserialize, Serialize};

use crate::ExecutorError;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum WalletError {
    InvalidPrivateKey
}

impl From<WalletError> for ExecutorError {
    fn from(value: WalletError) -> Self {
        ExecutorError::Wallet(value)
    }
}