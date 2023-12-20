use serde::{Deserialize, Serialize};
use crate::ExecutorError;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum GatewayError {
    CannotFetchAddressInfo { address: String },
    CannotParseAddressInfo { address: String },
    NoDataForAddressInfo { address: String },
    CannotFetchNetworkConfig,
    CannotParseNetworkConfig,
    CannotSimulateTransaction,
    CannotParseSimulationResponse,
}

impl From<GatewayError> for ExecutorError {
    fn from(value: GatewayError) -> Self {
        ExecutorError::Gateway(value)
    }
}