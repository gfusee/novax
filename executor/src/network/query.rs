use std::marker::PhantomData;
use async_trait::async_trait;
use multiversx_sc_scenario::multiversx_sc::codec::TopDecodeMulti;
use multiversx_sc_scenario::scenario_model::ScCallStep;
use multiversx_sdk::blockchain::CommunicationProxy;
use multiversx_sdk::data::address::Address;
use multiversx_sdk::data::vm::VmValueRequest;
use novax_data::{NativeConvertible, parse_query_return_string_data};
use crate::base::query::QueryExecutor;
use crate::error::executor::ExecutorError;
use crate::error::network::NetworkQueryError;
use crate::network::proxy::BlockchainProxy;
use crate::utils::transaction::data::SendableTransactionConvertible;

/// A convenient type alias for `QueryNetworkExecutor` with `CommunicationProxy` as the generic type.
pub type ProxyQueryExecutor = QueryNetworkExecutor<CommunicationProxy>;

/// A structure to execute smart contract queries on a real blockchain environment via a specified gateway.
///
/// This executor utilizes a blockchain proxy to communicate with the blockchain network and
/// execute the queries.
#[derive(Clone)]
pub struct QueryNetworkExecutor<Proxy: BlockchainProxy> {
    /// The URL of the gateway to the blockchain network.
    pub gateway_url: String,
    /// A phantom data field to keep the generic `Proxy` type.
    _data: PhantomData<Proxy>
}

impl<Proxy: BlockchainProxy> QueryNetworkExecutor<Proxy> {
    /// Constructs a new `QueryNetworkExecutor` with the specified gateway URL.
    ///
    /// # Parameters
    /// - `gateway_url`: The URL of the gateway to the blockchain network.
    ///
    /// # Returns
    /// A new instance of `QueryNetworkExecutor`.
    pub fn new(gateway_url: &str) -> Self {
        QueryNetworkExecutor {
            gateway_url: gateway_url.to_string(),
            _data: PhantomData
        }
    }
}

#[async_trait]
impl<Proxy: BlockchainProxy> QueryExecutor for QueryNetworkExecutor<Proxy> {
    /// Executes a smart contract query on the real blockchain environment.
    ///
    /// This method constructs a VM request from the provided `ScCallStep`, sends it to the blockchain network
    /// via the blockchain proxy, and processes the result to return it in a native format.
    async fn execute<OutputManaged>(&self, request: &ScCallStep) -> Result<OutputManaged::Native, ExecutorError>
        where
            OutputManaged: TopDecodeMulti + NativeConvertible
    {
        let sendable_tx = request.to_sendable_transaction();
        let receiver = Address::from_bech32_string(&sendable_tx.receiver).unwrap();
        let mut arguments: Vec<String> = sendable_tx.data.split('@').map(|e| e.to_string()).collect();

        if arguments.is_empty() {
            return Err(NetworkQueryError::EmptyArgs.into())
        }

        let function = arguments.remove(0);
        let vm_request = VmValueRequest {
            sc_address: receiver.clone(),
            func_name: function,
            caller: receiver,
            value: sendable_tx.egld_value.to_string(),
            args: arguments,
        };

        let blockchain = Proxy::new(&self.gateway_url);
        let result = blockchain.execute_vmquery(&vm_request).await?;

        let data: Vec<&str> = result.data.return_data.iter().map(AsRef::as_ref).collect();
        Ok(parse_query_return_string_data::<OutputManaged>(data.as_slice())?.to_native())
    }
}

#[async_trait]
impl QueryExecutor for &str {
    /// Allows using a string representing the gateway URL to execute a query on the real blockchain environment.
    ///
    /// This implementation creates a new `ProxyQueryExecutor` instance using the string as the gateway URL,
    /// and delegates the query execution to it.
    async fn execute<OutputManaged>(&self, request: &ScCallStep) -> Result<OutputManaged::Native, ExecutorError> where OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync {
        ProxyQueryExecutor::new(self).execute::<OutputManaged>(request).await
    }
}