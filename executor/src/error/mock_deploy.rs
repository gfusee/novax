use crate::error::executor::ExecutorError;
use serde::{Deserialize, Serialize};

/// An enumeration representing the various errors that can occur when deploying through the `MockExecutor`.
///
/// Currently, it includes the following variant:
/// * `WalletAddressNotPresent` - Indicates that a wallet address is not present but is required for the deployment operation.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum MockDeployError {
    /// This error variant is triggered when there is an attempt to deploy, but no wallet address is present.
    /// In the context of a `MockExecutor`, a caller's wallet address is essential for deployment, and if the
    /// `opt_caller` field is `None`, this error will be thrown.
    WalletAddressNotPresent,
}

/// An implementation of the `From` trait to allow for easy conversions from `MockDeployError` to `ExecutorError`.
///
/// This implementation facilitates the propagation of `MockDeployError`s through the code,
/// by allowing them to be converted into the more general `ExecutorError` type.
impl From<MockDeployError> for ExecutorError {
    /// Performs the conversion from a `MockDeployError` to an `ExecutorError`.
    ///
    /// # Parameters
    ///
    /// * `value`: The `MockDeployError` value to be converted.
    ///
    /// # Returns
    ///
    /// * `ExecutorError`: An `ExecutorError` instance containing the provided `MockDeployError` value.
    fn from(value: MockDeployError) -> Self {
        ExecutorError::MockDeploy(value)
    }
}