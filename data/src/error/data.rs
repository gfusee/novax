use crate::error::address::AddressError;
use serde::{Deserialize, Serialize};
use crate::error::utils::UtilsError;

/// Enumerates the centralized error handling types across different operations.
///
/// `DataError` serves as a unified error type that aggregates different error types from
/// various operations into a single enum. This centralization facilitates error handling
/// across different parts of the codebase.
///
/// # Variants
/// - `Address(AddressError)`: Encapsulates errors that occur during address-related operations,
///     as represented by the `AddressError` enum.
/// - `Utils(UtilsError)`: Encapsulates errors that occur within utility functions,
///     as represented by the `UtilsError` enum.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum DataError {
    /// Represents an error from address-related operations.
    Address(AddressError),
    /// Represents an error from utility functions.
    Utils(UtilsError),
}
