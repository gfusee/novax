use serde::{Deserialize, Serialize};
use novax::errors::NovaXError;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum TokenError {
    TokenNotFound { token_identifier: String },
    UnknownErrorForToken { token_identifier: String },
    UnknownErrorWhileGettingEsdtInfosOfAddress { address: String },
    CannotParseEsdtBalances { address: String },
    UnableToParseBigUintBalanceForTokenAndAddress { token_identifier: String, address: String, balance: String },
    NestedAppError(NovaXError)
}

impl From<NovaXError> for TokenError {
    fn from(value: NovaXError) -> Self {
        TokenError::NestedAppError(value)
    }
}