use serde::{Deserialize, Serialize};
use crate::error::data::DataError;

/// Enumerates the errors that can occur within utility functions.
///
/// This enum encapsulates specific error cases encountered in utility functions, such as
/// parsing and decoding operations. Each variant represents a distinct error case.
///
/// When a `UtilsError` occurs, it can be converted into a `DataError` which serves as
/// a centralized error type for broader error handling.
///
/// # Variants
/// - `CannotParseQueryResult`: This error occurs when it is impossible to decode the result
///     into a managed type, as encountered in functions like `parse_query_return_string_data`
///     and `parse_query_return_bytes_data`.
///
/// # Example
/// ```
/// # use novax_data::{UtilsError, DataError};
/// let error = UtilsError::CannotParseQueryResult;
/// let data_error: DataError = error.into();
/// ```
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum UtilsError {
    /// Represents an error case where the result cannot be decoded into a managed type.
    CannotParseQueryResult,
}

/// Provides a conversion from `UtilsError` to `DataError`.
///
/// This implementation allows for a `UtilsError` to be converted into a `DataError`,
/// facilitating centralized error handling.
impl From<UtilsError> for DataError {
    fn from(value: UtilsError) -> Self {
        DataError::Utils(value)
    }
}
