use std::collections::HashMap;
use serde::Deserialize;
use crate::network::models::generic::response::GatewayResponse;
use crate::TransactionOnNetworkTransactionLogs;

/// Type alias for `GatewayResponse` specialized for simulation responses.
/// It encapsulates the data specific to the simulation of blockchain transactions.
pub type SimulationGatewayResponse = GatewayResponse<SimulationGatewayResponseData>;

/// Struct representing the data part of the response from a transaction simulation request.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationGatewayResponseData {
    /// The total gas units consumed by the simulated transaction.
    pub tx_gas_units: u64,

    /// A message returned from the simulation, typically indicating the success or failure of the transaction.
    pub return_message: String,

    /// A collection of results from smart contracts invoked during the simulation.
    /// Each entry in the map corresponds to a smart contract result, keyed by a unique identifier.
    pub smart_contract_results: SimulationGatewayResponseDataScResults,

    /// A collection of logs from smart contracts invoked during the simulation.
    pub logs: Option<TransactionOnNetworkTransactionLogs>
}

/// Type alias for a map holding smart contract results as part of the simulation response.
/// Keys are unique identifiers (often transaction hashes) associated with each smart contract result.
pub type SimulationGatewayResponseDataScResults = HashMap<String, SimulationGatewayResponseDataScResultInfo>;

/// Struct representing detailed information about a smart contract's result in the simulation.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationGatewayResponseDataScResultInfo {
    /// The nonce of the transaction associated with the smart contract result.
    pub nonce: u64,

    /// The value transferred or involved in the smart contract call.
    pub value: u64,

    /// The receiver address in the smart contract call.
    pub receiver: String,

    /// The sender address initiating the smart contract call.
    pub sender: String,

    /// Additional data associated with the smart contract call, often in a specific encoded format.
    pub data: Option<String>,
}
