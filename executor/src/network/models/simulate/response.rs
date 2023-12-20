use std::collections::HashMap;
use serde::Deserialize;
use crate::network::models::generic::response::GatewayResponse;

pub type SimulationGatewayResponse = GatewayResponse<SimulationGatewayResponseData>;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationGatewayResponseData {
    pub tx_gas_units: u64,
    pub return_message: String,
    pub smart_contract_results: SimulationGatewayResponseDataScResults
}

pub type SimulationGatewayResponseDataScResults = HashMap<String, SimulationGatewayResponseDataScResultInfo>;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationGatewayResponseDataScResultInfo {
    pub nonce: u64,
    pub value: u64,
    pub receiver: String,
    pub sender: String,
    pub data: String,
}