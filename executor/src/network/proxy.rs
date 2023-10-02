use async_trait::async_trait;
use multiversx_sdk::blockchain::CommunicationProxy;
use multiversx_sdk::data::vm::{VmValueRequest, VmValuesResponseData};
use crate::error::executor::ExecutorError;
use crate::error::network::NetworkQueryError;

/// The `BlockchainProxy` trait provides an abstraction over the `CommunicationProxy` struct from the `multiversx-sdk` crate.
/// The main goal of this abstraction is to allow developers to mock the `CommunicationProxy` struct for testing purposes,
/// by providing a set of common behaviors required for interacting with the blockchain gateway.
#[async_trait]
pub trait BlockchainProxy: Clone + Send + Sync {
    /// Creates a new instance of a type implementing `BlockchainProxy`, initialized with the provided gateway URL.
    ///
    /// # Parameters
    ///
    /// * `gateway_url`: A string slice representing the URL of the blockchain gateway.
    ///
    /// # Returns
    ///
    /// * `Self`: A new instance of the implementing type.
    fn new(gateway_url: &str) -> Self;

    /// Asynchronously executes a virtual machine (VM) query on the blockchain, using the provided VM request.
    ///
    /// # Parameters
    ///
    /// * `vm_request`: A reference to a `VmValueRequest` object containing the details of the VM query.
    ///
    /// # Returns
    ///
    /// * `Result<VmValuesResponseData, ExecutorError>`: A result containing the VM query response data,
    ///   or an `ExecutorError` if the query fails.
    async fn execute_vmquery(
        &self,
        vm_request: &VmValueRequest,
    ) -> Result<VmValuesResponseData, ExecutorError>;
}

/// Implementation of the `BlockchainProxy` trait for the `CommunicationProxy` struct from the `multiversx-sdk` crate.
/// This implementation provides concrete methods to communicate with the blockchain gateway.
#[async_trait]
impl BlockchainProxy for CommunicationProxy {
    /// Creates a new `CommunicationProxy` instance using the specified blockchain gateway URL.
    ///
    /// # Parameters
    ///
    /// * `gateway_url`: A string slice representing the URL of the blockchain gateway.
    ///
    /// # Returns
    ///
    /// * `Self`: A new `CommunicationProxy` instance.
    fn new(gateway_url: &str) -> Self {
        CommunicationProxy::new(gateway_url.to_string())
    }

    /// Asynchronously executes a VM query on the blockchain using the provided VM request, and returns the response data.
    ///
    /// # Parameters
    ///
    /// * `vm_request`: A reference to a `VmValueRequest` object containing the details of the VM query.
    ///
    /// # Returns
    ///
    /// * `Result<VmValuesResponseData, ExecutorError>`: A result containing the VM query response data,
    ///   or an `ExecutorError` if the query fails.
    async fn execute_vmquery(&self, vm_request: &VmValueRequest) -> Result<VmValuesResponseData, ExecutorError> {
        let result = self.execute_vmquery(vm_request).await;

        if let Err(error) = result {
            let message = error.to_string();
            return Err(NetworkQueryError::ErrorWhileSendingRequest { message }.into())
        }

        Ok(result.unwrap())
    }
}
