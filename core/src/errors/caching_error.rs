use serde::{Deserialize, Serialize};
use crate::errors::novax_error::NovaXError;

/// An enumeration of errors that may occur during caching operations within the contract framework.
/// These errors may arise when attempting to serialize or deserialize data, or when encountering issues with the getter method.
///
/// # Variants
///
/// - `UnableToSerialize`: This variant is triggered when serialization of data fails. This may be due to
///   data that does not meet the requirements for serialization.
///
/// - `UnableToDeserialize`: This variant is triggered when deserialization of data fails. This may be
///   due to malformed data or data that does not conform to the expected structure.
///
/// - `ErrorInGetter`: This variant represents an error that occurs when attempting to retrieve data
///   using a getter method. This could be due to a logic error or an unexpected condition within the getter method.
///
/// - `UnknownError`: This variant represents an unspecified error that may occur during caching operations.
///   This is a catch-all error variant for unexpected or unknown issues.
///
/// # Conversion to NovaXError
///
/// This enum implements a conversion to `NovaXError`, a more general error type, via the `From` trait.
/// This allows for easy error propagation with the `?` operator and better integration with the overarching error handling framework.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum CachingError {
    /// Encountered when there is a failure in serializing data before caching it.
    /// This could be due to the data being in a format that's not serializable or other unforeseen issues.
    UnableToSerialize,

    /// Encountered when there is a failure in deserializing data retrieved from the cache.
    /// This could be due to the data being corrupted or not in the expected format.
    UnableToDeserialize,

    /// Encountered when an error occurs in the getter function which is used to fetch data in case
    /// it is not found in the cache.
    ErrorInGetter,

    /// A catch-all for other unforeseen errors that may occur during caching operations.
    UnknownError,
}

impl From<CachingError> for NovaXError {
    /// Converts a `CachingError` into a `NovaXError`.
    ///
    /// This conversion wraps the `CachingError` variant in the `NovaXError::Caching` variant,
    /// allowing it to be handled as part of a more general error handling system.
    fn from(value: CachingError) -> Self {
        NovaXError::Caching(value)
    }
}