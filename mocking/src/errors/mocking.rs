use serde::{Deserialize, Serialize};
use novax::errors::NovaXError;
use novax_token::error::token::TokenError;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum NovaXMockingError {
    UnableToFetchAddressKeys,
    UnableToParseAddressKeys,
    UnableToReadInfosFromFile,
    NovaXError(NovaXError),
    NovaXTokenError(TokenError)
}

impl From<NovaXError> for NovaXMockingError {
    fn from(value: NovaXError) -> Self {
        NovaXMockingError::NovaXError(value)
    }
}

impl From<TokenError> for NovaXMockingError {
    fn from(value: TokenError) -> Self {
        NovaXMockingError::NovaXTokenError(value)
    }
}