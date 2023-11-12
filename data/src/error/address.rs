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
/// - `InvalidBech32String`: Represents an error case where an invalid Bech32 string is provided.
/// - `CannotConvertToBech32String`: Represents an error case where an `Address` cannot be converted to its
///   Bech32 string representation.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum AddressError {
    /// Represents an error case where an invalid Bech32 string is provided.
    InvalidBech32String {
        /// The invalid Bech32 string that led to the error. Storing this string helps in diagnosing
        /// the specific input that failed to parse, facilitating easier troubleshooting.
        invalid_value: String
    },
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
