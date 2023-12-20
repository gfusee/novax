use serde::{Deserialize, Serialize};
use crate::network::models::generic::response::GatewayResponse;

pub type AddressGatewayResponse = GatewayResponse<AddressGatewayInfo>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddressGatewayInfo {
    pub account: AddressGatewayInfoAccount
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddressGatewayInfoAccount {
    pub address: String,
    pub nonce: u64,
    pub balance: String
}