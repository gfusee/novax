use crate::error::data::DataError;
use serde::{Deserialize, Serialize};

/// Enumerates the errors that can occur within operations related to the `Address` struct.
///
/// This enum encapsulates specific error cases encountered when working with the `Address`
/// struct, such as converting to and from Bech32 string representation.
///
/// When an `AddressError` occurs, it can be converted into a `DataError` which serves as
/// a centralized error type for broader error handling.
///
/// # Variants
/// - `InvalidBech32String`: This error occurs when attempting to construct an `Address`
///     from an invalid Bech32 string.
/// - `CannotConvertToBech32String`: This error occurs when attempting to convert an `Address`
///     to its Bech32 string representation.
///
/// # Example
/// ```
/// # use novax_data::{AddressError, DataError};
/// let error = AddressError::InvalidBech32String;
/// let data_error: DataError = error.into();
/// ```
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum AddressError {
    /// Represents an error case where an invalid Bech32 string is provided.
    InvalidBech32String { invalid_value: String },
    /// Represents an error case where an `Address` cannot be converted to its Bech32 string representation.
    CannotConvertToBech32String,
}

/// Provides a conversion from `AddressError` to `DataError`.
///
/// This implementation allows for an `AddressError` to be converted into a `DataError`,
/// facilitating centralized error handling.
impl From<AddressError> for DataError {
    fn from(value: AddressError) -> Self {
        DataError::Address(value)
    }
}
