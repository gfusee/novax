use serde::{Deserialize, Serialize};
use novax::errors::NovaXError;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum AccountError {
    AccountNotFound { address: String },
    CannotDecodeCodeMetadata { metadata: String},
    UnknownErrorWhileGettingInfosOfAccount { address: String},
    CannotParseAccountInfo { address: String},
    CannotParseAccountBalance { address: String, balance: String},
    CannotParseAccountDeveloperReward { address: String, reward: String},
    CannotParseAccountOwnerAddress { address: String, owner: String},
    NestedAppError(NovaXError)
}

impl From<NovaXError> for AccountError {
    fn from(value: NovaXError) -> Self {
        AccountError::NestedAppError(value)
    }
}