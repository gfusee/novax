use serde::{Deserialize, Serialize};

use crate::ExecutorError;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum DateError {
    UnableToGetCurrentTimestamp
}

impl From<DateError> for ExecutorError {
    fn from(value: DateError) -> Self {
        ExecutorError::Date(value)
    }
}