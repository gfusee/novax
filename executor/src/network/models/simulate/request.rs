use serde::Serialize;

pub struct SimulationGatewayRequest {
    pub value: String,
    pub receiver: String,
    pub sender: String,
    pub gas_limit: u64,
    pub data: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationGatewayRequestBody {
    pub nonce: u64,
    pub value: String,
    pub receiver: String,
    pub sender: String,
    pub gas_price: u64,
    pub gas_limit: u64,
    pub data: String,
    pub chain_id: String,
    pub version: u8
}