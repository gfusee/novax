use crate::error::executor::ExecutorError;
use serde::{Deserialize, Serialize};

/// An enumeration representing the various errors that can occur when deploying through the `MockExecutor`.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum MockDeployError {
    /// This error variant is triggered when there is an attempt to deploy, but no caller address is present.
    /// In the context of a `MockExecutor`, a caller's address is essential for deployment, and if the
    /// `opt_caller` field is `None`, this error will be thrown.
    CallerAddressNotPresent,
}

/// An implementation of the `From` trait to allow for easy conversions from `MockDeployError` to `ExecutorError`.
///
/// This implementation facilitates the propagation of `MockDeployError`s through the code,
/// by allowing them to be converted into the more general `ExecutorError` type.
impl From<MockDeployError> for ExecutorError {
    /// Performs the conversion from a `MockDeployError` to an `ExecutorError`.
    fn from(value: MockDeployError) -> Self {
        ExecutorError::MockDeploy(value)
    }
}