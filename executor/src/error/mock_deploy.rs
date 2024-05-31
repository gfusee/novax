use crate::error::executor::ExecutorError;
use serde::{Deserialize, Serialize};

/// TODO An enumeration representing the various errors that can occur when deploying through the `MockExecutor`.
///
/// Currently, it includes the following variant:
/// * `CallerAddressNotPresent` - Indicates that a caller address is not present but is required for the deployment operation.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum MockDeployError {
    /// TODO This error variant is triggered when there is an attempt to deploy, but no caller address is present.
    /// In the context of a `MockExecutor`, a caller's address is essential for deployment, and if the
    /// `opt_caller` field is `None`, this error will be thrown.
    CallerAddressNotPresent,
}

/// TODO An implementation of the `From` trait to allow for easy conversions from `MockDeployError` to `ExecutorError`.
///
/// This implementation facilitates the propagation of `MockDeployError`s through the code,
/// by allowing them to be converted into the more general `ExecutorError` type.
impl From<MockDeployError> for ExecutorError {
    /// TODO Performs the conversion from a `MockDeployError` to an `ExecutorError`.
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