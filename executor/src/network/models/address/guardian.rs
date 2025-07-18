use crate::network::models::generic::response::GatewayResponse;
use serde::{Deserialize, Serialize};

pub type AddressGuardianDataGatewayResponse = GatewayResponse<AddressGatewayGuardianData>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddressGatewayGuardianInfo {
    pub address: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GuardianData {
    pub active_guardian: Option<AddressGatewayGuardianInfo>,
    pub pending_guardian: Option<AddressGatewayGuardianInfo>,
    pub guarded: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddressGatewayGuardianData {
    pub guardian_data: Option<GuardianData>,
}