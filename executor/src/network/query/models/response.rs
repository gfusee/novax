use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmValuesQueryResponse {
    pub data: Option<VmValuesQueryResponseData>,
    pub error: String
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