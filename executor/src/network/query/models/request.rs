use serde::{Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VmValuesQueryRequest {
    pub sc_address: String,
    pub func_name: String,
    pub args: Vec<String>,
    pub caller: Option<String>,
    pub value: Option<String>
}