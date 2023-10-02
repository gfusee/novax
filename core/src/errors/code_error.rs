use serde::{Deserialize, Serialize};
use crate::errors::novax_error::NovaXError;

/// An enumeration of errors that may occur when working with smart contract code.
/// This error type is specific to operations like reading the contract code from a file.
///
/// # Variants
///
/// - `UnableToReadCodeFromFile`: This variant represents an error that occurs when the system is
///   unable to read the smart contract code from a specified file. This might be due to the file
///   not existing, permissions issues, or other file system related errors.
///
/// # Conversion to NovaXError
///
/// This enum implements a conversion to `NovaXError`, a more general error type, via the `From` trait.
/// This allows for easy error propagation with the `?` operator.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum CodeError {
    /// This error occurs when the system is unable to read contract code from a file.
    UnableToReadCodeFromFile,
}

impl From<CodeError> for NovaXError {
    /// Converts a `CodeError` into a `NovaXError`.
    ///
    /// This conversion wraps the `CodeError` variant in the `NovaXError::Code` variant,
    /// allowing it to be handled as part of a more general error handling system.
    fn from(value: CodeError) -> Self {
        NovaXError::Code(value)
    }
}