use multiversx_sdk::data::vm::VmValuesResponseData;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmValuesQueryResponse {
    pub data: Option<VmValuesResponseData>
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmValuesQueryResponseData {
    pub data: VmValuesQueryResponseDataData
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmValuesQueryResponseDataData {
    pub return_data: Vec<String>
}