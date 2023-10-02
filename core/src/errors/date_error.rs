use serde::{Deserialize, Serialize};
use crate::errors::novax_error::NovaXError;

/// An enumeration of errors that may arise within the "novax-caching" crate during operations related to date and time.
/// This particular error type is a temporary workaround and there are plans to change or refine it in future iterations.
///
/// # Variants
///
/// - `UnableToGetCurrentTimestamp`: This variant is triggered when the operation to fetch the current timestamp fails.
///   The underlying cause may be due to issues with the system clock, permissions, or other unforeseen circumstances.
///
/// # Conversion to NovaXError
///
/// This enum implements a conversion to `NovaXError`, a more general error type, via the `From` trait.
/// This allows for easy error propagation with the `?` operator and better integration with the overarching error handling framework.
///
/// # Note
///
/// This error type resides solely within the "novax-caching" crate and is used as a temporary measure to handle
/// timestamp-related issues. It is intended to be revised or replaced in future releases.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum DateError {
    /// This error is thrown when the system is unable to retrieve the current timestamp.
    /// This could be due to system-level restrictions or other unforeseen issues.
    UnableToGetCurrentTimestamp,
}

impl From<DateError> for NovaXError {
    /// Converts a `DateError` into a `NovaXError`.
    ///
    /// This conversion wraps the `DateError` variant in the `NovaXError::Date` variant,
    /// allowing it to be handled as part of a more general error handling system.
    fn from(value: DateError) -> Self {
        NovaXError::Date(value)
    }
}