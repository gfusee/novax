use serde::{Deserialize, Serialize};
use novax::errors::NovaXError;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum NovaXMockingError {
    UnableToFetchAddressKeys,
    UnableToParseAddressKeys,
    UnableToReadInfosFromFile,
    NovaXError(NovaXError)
}

impl From<NovaXError> for NovaXMockingError {
    fn from(value: NovaXError) -> Self {
        NovaXMockingError::NovaXError(value)
    }
}