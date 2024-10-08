use std::sync::Arc;
use async_trait::async_trait;
use multiversx_sc_scenario::multiversx_sc::codec::TopDecodeMulti;
use num_bigint::BigUint;
use tokio::sync::Mutex;
use novax_data::{Address, NativeConvertible};
use crate::error::executor::ExecutorError;
use crate::TokenTransfer;

/// A trait representing the execution of smart contract queries.
///
/// This trait is implemented by types that can execute smart contract queries in a specific environment,
/// like a real blockchain or a mocked one. The trait's associated function, `execute`, is responsible for
/// sending a query request, executing the query on the blockchain or mocked environment,
/// and returning the result of the query.
#[async_trait]
pub trait QueryExecutor: Send + Sync {
    /// Executes a smart contract query and returns the result.
    async fn execute<OutputManaged>(
        &self,
        to: &Address,
        function: String,
        arguments: Vec<Vec<u8>>,
        egld_value: BigUint,
        esdt_transfers: Vec<TokenTransfer>
    ) -> Result<OutputManaged::Native, ExecutorError>
        where
            OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync;
}

/// An implementation of `QueryExecutor` for `Arc<T>` where `T: QueryExecutor`.
///
/// This implementation allows shared access to an executor instance.
#[async_trait]
impl<T: QueryExecutor> QueryExecutor for Arc<T> {
    async fn execute<OutputManaged>(
        &self,
        to: &Address,
        function: String,
        arguments: Vec<Vec<u8>>,
        egld_value: BigUint,
        esdt_transfers: Vec<TokenTransfer>
    ) -> Result<OutputManaged::Native, ExecutorError>
        where
            OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
    {
        T::execute::<OutputManaged>(
            self,
            to,
            function,
            arguments,
            egld_value,
            esdt_transfers
        ).await
    }
}

/// An implementation of `QueryExecutor` for `Arc<Mutex<T>>` where `T: QueryExecutor`.
///
/// This implementation allows exclusive access to an executor instance, ensuring safe mutable access.
#[async_trait]
impl<T: QueryExecutor> QueryExecutor for Arc<Mutex<T>> {
    async fn execute<OutputManaged>(
        &self,
        to: &Address,
        function: String,
        arguments: Vec<Vec<u8>>,
        egld_value: BigUint,
        esdt_transfers: Vec<TokenTransfer>
    ) -> Result<OutputManaged::Native, ExecutorError>
        where
            OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
    {
        {
            let executor = self.lock().await;
            executor.execute::<OutputManaged>(
                to,
                function,
                arguments,
                egld_value,
                esdt_transfers
            ).await
        }
    }
}