use crate::error::network::NetworkQueryError;
use serde::{Deserialize, Serialize};
use novax_data::DataError;
use crate::error::mock_deploy::MockDeployError;

/// An enumeration representing the various types of errors that can be encountered within the executor context.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum ExecutorError {
    /// This variant wraps errors encountered during network queries, which may include issues such as connection
    /// failures or malformed requests. The wrapped `NetworkQueryError` provides more detailed information about
    /// the nature of the network-related error that occurred.
    NetworkQuery(NetworkQueryError),

    /// Wraps errors related to data operations, usually arising from the `novax-data` crate. These may include errors
    /// related to data parsing, validation, or any other data-related operation. The wrapped `DataError` provides
    /// more detailed information about the nature of the data-related error that occurred.
    DataError(DataError),

    /// This variant wraps errors encountered during mock deployments. This is particularly useful when using the
    /// `MockExecutor` for testing or simulation purposes. The wrapped `MockDeployError` provides more detailed
    /// information about the nature of the mock deployment-related error that occurred.
    MockDeploy(MockDeployError),
}

/// An implementation of the `From` trait to allow for easy conversions from `DataError` to `ExecutorError`.
impl From<DataError> for ExecutorError {
    fn from(value: DataError) -> Self {
        ExecutorError::DataError(value)
    }
}