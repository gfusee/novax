use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct TokenPropertiesResponseDataData {
    #[serde(rename = "returnData")]
    pub return_data: Option<[String; 18]>,
    #[serde(rename = "returnMessage")]
    pub return_message: Option<String>
}

#[derive(Deserialize)]
pub(crate) struct TokenPropertiesResponseData {
    pub data: TokenPropertiesResponseDataData,
}

#[derive(Deserialize)]
pub(crate) struct TokenPropertiesResponse {
    pub data: TokenPropertiesResponseData
}
