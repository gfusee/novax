use std::marker::PhantomData;

use async_trait::async_trait;
use multiversx_sc::codec::TopDecodeMulti;
use num_bigint::BigUint;

use novax_data::{NativeConvertible, parse_query_return_string_data};

use crate::{BlockchainProxy, ExecutorError, QueryExecutor, TokenTransfer, VmValuesQueryRequest};
use crate::network::query::proxy::NetworkBlockchainProxy;

/// A convenient type alias for `QueryNetworkExecutor` with `NetworkBlockchainProxy` as the generic type.
pub type ProxyQueryExecutor = QueryNetworkExecutor<NetworkBlockchainProxy>;

/// A structure to execute smart contract queries on a real blockchain environment via a specified gateway.
///
/// This executor utilizes a blockchain proxy to communicate with the blockchain network and
/// execute the queries.
#[derive(Clone, Debug)]
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
    pub fn new(gateway_url: String) -> Self {
        QueryNetworkExecutor {
            gateway_url,
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
    async fn execute<OutputManaged>(
        &self,
        to: &novax_data::Address,
        function: String,
        arguments: &[Vec<u8>],
        egld_value: BigUint,
        esdt_transfers: Vec<TokenTransfer>
    ) -> Result<OutputManaged::Native, ExecutorError>
        where
            OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
    {
        let sc_address = to.to_bech32_string()?;
        let arguments = encode_arguments(arguments);

        let vm_request = VmValuesQueryRequest { // TODO: put this in a separate function so normalization can be tested
            sc_address: sc_address.clone(),
            func_name: function,
            caller: None, // TODO
            value: None, // TODO: normalize
            args: arguments,
        };

        let blockchain = Proxy::new(self.gateway_url.clone());
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
    async fn execute<OutputManaged>(
        &self,
        to: &novax_data::Address,
        function: String,
        arguments: &[Vec<u8>],
        egld_value: BigUint,
        esdt_transfers: Vec<TokenTransfer>
    ) -> Result<OutputManaged::Native, ExecutorError>
        where
            OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
    {
        self.to_string()
            .execute::<OutputManaged>(
                to,
                function,
                arguments,
                egld_value,
                esdt_transfers
            )
            .await
    }
}

#[async_trait]
impl QueryExecutor for String {
    /// Allows using a string representing the gateway URL to execute a query on the real blockchain environment.
    ///
    /// This implementation creates a new `ProxyQueryExecutor` instance using the string as the gateway URL,
    /// and delegates the query execution to it.
    async fn execute<OutputManaged>(
        &self,
        to: &novax_data::Address,
        function: String,
        arguments: &[Vec<u8>],
        egld_value: BigUint,
        esdt_transfers: Vec<TokenTransfer>
    ) -> Result<OutputManaged::Native, ExecutorError>
        where
            OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
    {
        ProxyQueryExecutor::new(self.to_string())
            .execute::<OutputManaged>(
                to,
                function,
                arguments,
                egld_value,
                esdt_transfers
            )
            .await
    }
}

fn encode_arguments(arguments: &[Vec<u8>]) -> Vec<String> {
    arguments.iter()
        .map(|arg| hex::encode(arg))
        .collect()
}

#[cfg(test)]
mod tests {
    use multiversx_sc::codec::TopEncode;
    use multiversx_sc::imports::ManagedVec;
    use multiversx_sc::types::ManagedBuffer;
    use multiversx_sc_scenario::imports::StaticApi;

    use crate::network::query::executor::encode_arguments;

    #[test]
    fn test_encode_arguments_empty() {
        let result = encode_arguments(&vec![]);
        let expected: Vec<String> = vec![];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_encode_one_type() {
        let vec: ManagedVec<StaticApi, ManagedBuffer<StaticApi>> = ManagedVec::from_single_item(ManagedBuffer::from("Hey!"));

        let mut arguments: Vec<Vec<u8>> = vec![];
        for item in vec.into_iter() {
            let mut encoded_buffer: ManagedBuffer<StaticApi> = ManagedBuffer::new();
            _ = item.top_encode(&mut encoded_buffer);

            arguments.push(encoded_buffer.to_boxed_bytes().into_vec());
        }

        let result = encode_arguments(&arguments);
        let expected = vec!["48657921".to_string()];

        assert_eq!(result, expected)
    }

    #[test]
    fn test_encode_two_type() {
        let mut vec: ManagedVec<StaticApi, ManagedBuffer<StaticApi>> = ManagedVec::new();
        vec.push(ManagedBuffer::from("Hey!"));
        vec.push(ManagedBuffer::from("Hi!"));

        let mut arguments: Vec<Vec<u8>> = vec![];
        for item in vec.into_iter() {
            let mut encoded_buffer: ManagedBuffer<StaticApi> = ManagedBuffer::new();
            _ = item.top_encode(&mut encoded_buffer);

            arguments.push(encoded_buffer.to_boxed_bytes().into_vec());
        }

        let result = encode_arguments(&arguments);
        let expected = vec!["48657921".to_string(), "486921".to_string()];

        assert_eq!(result, expected)
    }

}