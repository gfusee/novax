use serde::{Deserialize, Serialize};
use crate::errors::novax_error::NovaXError;

/// An enumeration of errors that may occur during account-related operations within the contract framework.
/// These errors primarily arise during the fetching and parsing of account information.
///
/// # Variants
///
/// - `CannotFetchAccountInfos`: This variant is triggered when there's a failure in fetching account information.
///   This could be due to network issues, incorrect configurations, or other unforeseen circumstances that prevent
///   the retrieval of account data.
///
/// - `CannotParseAccountInfos`: This variant is triggered when there's a failure in parsing the fetched account information.
///   This could occur if the data structure of the account information has changed or if there's a bug in the parsing logic.
///
/// # Conversion to NovaXError
///
/// This enum implements a conversion to `NovaXError`, a more general error type, via the `From` trait.
/// This allows for easy error propagation with the `?` operator and better integration with the overarching error handling framework.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum AccountError {
    /// Encountered when there is a failure in fetching account information from a source.
    /// This could be due to network issues, incorrect credentials, or other unforeseen problems.
    CannotFetchAccountInfos,

    /// Encountered when there is a failure in parsing the account information retrieved.
    /// This could be due to the data being corrupted, or not in the expected format.
    CannotParseAccountInfos,
}

impl From<AccountError> for NovaXError {
    /// Converts an `AccountError` into a `NovaXError`.
    ///
    /// This conversion wraps the `AccountError` variant in the `NovaXError::Account` variant,
    /// allowing it to be handled as part of a more general error handling system.
    fn from(value: AccountError) -> Self {
        NovaXError::Account(value)
    }
}