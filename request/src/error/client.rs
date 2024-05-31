use serde::{Deserialize, Serialize};
use crate::error::request::RequestError;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum ClientError {
    UnknownError
}

impl From<ClientError> for RequestError {
    fn from(value: ClientError) -> Self {
        RequestError::Client(value)
    }
}