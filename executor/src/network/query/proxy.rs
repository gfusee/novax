use async_trait::async_trait;
use reqwest::Client;

use crate::error::executor::ExecutorError;
use crate::network::query::models::request::VmValuesQueryRequest;
use crate::network::query::models::response::{VmValuesQueryResponse, VmValuesQueryResponseData};
use crate::NetworkQueryError;

#[async_trait]
pub trait BlockchainProxy: Send + Sync {
    fn new(gateway_url: String) -> Self;

    async fn execute_vmquery(
        &self,
        vm_request: &VmValuesQueryRequest,
    ) -> Result<VmValuesQueryResponseData, ExecutorError>;
}

pub struct NetworkBlockchainProxy {
    pub gateway_url: String
}

#[async_trait]
impl BlockchainProxy for NetworkBlockchainProxy {
    fn new(gateway_url: String) -> Self {
        Self {
            gateway_url,
        }
    }

    async fn execute_vmquery(&self, vm_request: &VmValuesQueryRequest) -> Result<VmValuesQueryResponseData, ExecutorError> {
        let url = format!("{}/vm-values/query", self.gateway_url);

        let json = serde_json::to_string(vm_request)
            .map_err(|_| NetworkQueryError::CannotSerializeVmValuesRequestBody)?;

        let result = match Client::new()
            .post(url)
            .body(json.clone())
            .send()
            .await
        {
            Ok(result) => result,
            Err(error) => return Err(NetworkQueryError::ErrorWhileSendingRequest { request_body: json, message: error.to_string() }.into())
        };

        let text = match result
            .text()
            .await
        {
            Ok(result) => result,
            Err(error) => return Err(NetworkQueryError::ErrorWhileSendingRequest { request_body: json, message: error.to_string() }.into())
        };

        let response = serde_json::from_str::<VmValuesQueryResponse>(&text)
            .map_err(|_| NetworkQueryError::CannotDeserializeVmValuesResponse)?;

        let Some(response_data) = response.data else {
            return Err(NetworkQueryError::ErrorInResponse { message: response.error }.into())
        };

        Ok(response_data)
    }
}