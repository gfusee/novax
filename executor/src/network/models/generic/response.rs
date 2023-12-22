use serde::{Deserialize, Serialize};

/// Represents a generic response format from the MultiversX gateway. This structure is used for parsing responses
/// from various gateway endpoints. It's a generic struct that can accommodate different types of data in its response.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GatewayResponse<T> {
    /// An optional field containing the actual data returned from the gateway. The type `T` is generic and
    /// can represent any struct or data format expected in the response. If the request to the gateway fails,
    /// or if there's no specific data to return, this field will be `None`.
    pub data: Option<T>,

    /// A string describing any error that occurred during the gateway interaction. In successful responses,
    /// this field is typically an empty string. In case of an error, it provides a description of the issue.
    pub error: String,

    /// A code associated with the response. This can be used to programmatically identify the type of response
    /// or error received from the gateway. For successful requests, this usually contains a success indicator,
    /// while for failed requests, it contains an error code.
    pub code: String,
}
