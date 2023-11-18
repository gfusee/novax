use serde::{Deserialize, Serialize};
use crate::errors::novax_error::NovaXError;

/// Enumerates coding-related errors, specifically for encoding and decoding processes.
///
/// This enum represents errors that can occur during the encoding and decoding stages, particularly
/// involving serialization and deserialization of data structures used within the NovaX framework.
///
/// # Variants
/// - `CannotDecodeEsdtAttributes`: This error occurs when there's a failure in decoding attributes
///   associated with an ESDT (Elrond Standard Digital Token). While attributes are typically found in
///   non-fungible tokens (NFTs), this error covers scenarios where decoding such attributes fails for
///   any ESDT, fungible or non-fungible.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum CodingError {
    /// Represents an error that occurs when the decoding of attributes for an ESDT (Elrond Standard Digital Token)
    /// fails. This error is particularly significant in the context of non-fungible tokens (NFTs), where attributes
    /// play a crucial role in defining the token's properties and metadata.
    CannotDecodeEsdtAttributes,
}

impl From<CodingError> for NovaXError {
    /// Converts a `CodingError` into a `NovaXError`.
    ///
    /// This implementation enables the seamless transformation of a specific coding error into the broader
    /// `NovaXError` type. This is particularly useful for error handling strategies that require a consistent
    /// error type across different modules of the NovaX framework.
    ///
    /// # Arguments
    /// - `value`: The `CodingError` instance to be converted.
    ///
    /// # Returns
    /// A `NovaXError` instance, specifically as a `NovaXError::Coding` variant containing the original `CodingError`.
    fn from(value: CodingError) -> Self {
        NovaXError::Coding(value)
    }
}