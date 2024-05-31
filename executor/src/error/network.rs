use crate::error::executor::ExecutorError;
use serde::{Deserialize, Serialize};

/// TODO An enumeration representing the various errors that can occur during network query operations.
///
/// It includes the following variants:
/// * `EmptyArgs` - Indicates that the arguments provided for the network query are empty when they shouldn't be.
/// * `ErrorWhileSendingRequest` - Indicates an error occurred while sending the request to the network.
///   The accompanying message provides more detail regarding the nature of the error.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum NetworkQueryError {
    /// This error variant is triggered when the argument provided for the network query is empty,
    /// which is not acceptable for the network query operation to proceed.
    EmptyArgs,
    CannotSerializeVmValuesRequestBody,
    CannotDeserializeVmValuesResponse,
    /// This error variant is triggered when there is an error while sending a request to the network.
    /// The exact error message is encapsulated in the `message` field.
    ErrorWhileSendingRequest {
        /// Contains a descriptive error message explaining the reason for the failure while sending the request.
        message: String
    },
    ErrorInResponse {
        message: String
    }
}

/// An implementation of the `From` trait to allow for easy conversions from `NetworkQueryError` to `ExecutorError`.
///
/// This implementation facilitates the propagation of `NetworkQueryError`s through the code,
/// by allowing them to be converted into the more general `ExecutorError` type.
impl From<NetworkQueryError> for ExecutorError {
    /// Performs the conversion from a `NetworkQueryError` to an `ExecutorError`.
    ///
    /// # Parameters
    ///
    /// * `value`: The `NetworkQueryError` value to be converted.
    ///
    /// # Returns
    ///
    /// * `ExecutorError`: An `ExecutorError` instance containing the provided `NetworkQueryError` value.
    fn from(value: NetworkQueryError) -> Self {
        ExecutorError::NetworkQuery(value)
    }
}