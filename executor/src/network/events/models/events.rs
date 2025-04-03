use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElasticSearchEvent {
    pub log_address: String,
    pub identifier: String,
    pub address: String,
    pub data: String,
    pub topics: Vec<String>,
    pub timestamp: u64,
}