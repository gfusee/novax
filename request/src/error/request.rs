use serde::{Deserialize, Serialize};
use crate::error::client::ClientError;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum RequestError {
   Client(ClientError)
}