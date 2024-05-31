use crate::error::executor::ExecutorError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum MockTransactionError {
    /// This error variant is triggered when there is an attempt to deploy, but no caller address is present.
    /// In the context of a `MockExecutor`, a caller's wallet address is essential for calling a contract, and if the
    /// `opt_caller` field is `None`, this error will be thrown.
    CallerAddressNotPresent,
}

/// An implementation of the `From` trait to allow for easy conversions from `MockTransactionError` to `ExecutorError`.
///
/// This implementation facilitates the propagation of `MockTransactionError`s through the code,
/// by allowing them to be converted into the more general `ExecutorError` type.
impl From<MockTransactionError> for ExecutorError {
    /// Performs the conversion from a `MockTransactionError` to an `ExecutorError`.
    ///
    /// # Parameters
    ///
    /// * `value`: The `MockTransactionError` value to be converted.
    ///
    /// # Returns
    ///
    /// * `ExecutorError`: An `ExecutorError` instance containing the provided `MockTransactionError` value.
    fn from(value: MockTransactionError) -> Self {
        ExecutorError::MockTransaction(value)
    }
}