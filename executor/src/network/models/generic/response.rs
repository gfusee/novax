use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GatewayResponse<T>
{
    pub data: Option<T>,
    pub error: String,
    pub code: String
}