use serde::{Deserialize, Serialize};
use crate::network::models::generic::response::GatewayResponse;

pub type NetworkGatewayConfigResponse = GatewayResponse<NetworkGatewayConfig>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkGatewayConfig {
    pub config: NetworkGatewayConfigData
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkGatewayConfigData {
    pub erd_chain_id: String,
    pub erd_min_gas_price: u64,
    pub erd_min_transaction_version: u8,
}