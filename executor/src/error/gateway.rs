use serde::{Deserialize, Serialize};
use crate::ExecutorError;

/// An enumeration representing various types of errors that can occur during interactions with the MultiversX gateway.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum GatewayError {
    /// Error that occurs when fetching information for an address from the `/address/{address}` endpoint.
    CannotFetchAddressInfo {
        /// The blockchain address for which the information fetch operation failed.
        address: String
    },

    /// Represents an error when parsing the address information fetched from the gateway.
    CannotParseAddressInfo {
        /// The blockchain address whose information encountered a parsing error.
        address: String
    },

    /// Indicates that no data was available for the requested address information.
    NoDataForAddressInfo {
        /// The blockchain address for which the gateway's response lacked necessary details.
        address: String
    },

    /// Error encountered when attempting to fetch the network configuration from the `/network/config` endpoint.
    CannotFetchNetworkConfig,

    /// Occurs when there is a problem parsing the network configuration data fetched from the gateway.
    CannotParseNetworkConfig,

    /// Represents an error when simulating a transaction through the gateway.
    CannotSimulateTransaction,

    /// This error is thrown when there is a problem parsing the response from a transaction simulation.
    CannotParseSimulationResponse,
}

impl From<GatewayError> for ExecutorError {
    fn from(value: GatewayError) -> Self {
        ExecutorError::Gateway(value)
    }
}
