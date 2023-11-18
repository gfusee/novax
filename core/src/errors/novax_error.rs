use serde::{Deserialize, Serialize};
use novax_data::DataError;
use novax_executor::ExecutorError;
use crate::errors::account_error::AccountError;
use crate::errors::caching_error::CachingError;
use crate::errors::code_error::CodeError;
use crate::errors::CodingError;
use crate::errors::date_error::DateError;

/// The main error type for the `novax` crate.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum NovaXError {
    /// Errors related to data processing or handling.
    Data(DataError),
    /// Errors related to the execution of transactions or queries.
    Executor(ExecutorError),
    /// Errors related to caching mechanisms.
    Caching(CachingError),
    /// Errors related to obtaining the current timestamp.
    /// This is a temporary workaround, and changes are planned for this error type.
    Date(DateError),
    /// Errors related to fetching or parsing account information.
    Account(AccountError),
    /// Errors occurring during the encoding or decoding of managed types. This variant encompasses issues
    /// related to serialization and deserialization processes, which are fundamental in ensuring data
    /// integrity and adherence to expected formats.
    Coding(CodingError),
    /// Errors related to reading contract code from a file.
    Code(CodeError),
}

impl From<DataError> for NovaXError {
    fn from(value: DataError) -> Self {
        NovaXError::Data(value)
    }
}

impl From<ExecutorError> for NovaXError {
    fn from(value: ExecutorError) -> Self {
        NovaXError::Executor(value)
    }
}