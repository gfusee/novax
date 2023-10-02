use std::sync::Arc;
use async_trait::async_trait;
use multiversx_sc_scenario::multiversx_sc::codec::TopDecodeMulti;
use multiversx_sc_scenario::scenario_model::ScCallStep;
use tokio::sync::Mutex;
use novax_data::NativeConvertible;
use crate::error::executor::ExecutorError;

/// A trait representing the execution of smart contract queries.
///
/// This trait is implemented by types that can execute smart contract queries in a specific environment,
/// like a real blockchain or a mocked one. The trait's associated function, `execute`, is responsible for
/// sending a query request, executing the query on the blockchain or mocked environment,
/// and returning the result of the query.
#[async_trait]
pub trait QueryExecutor: Clone + Send + Sync {
    /// Executes a smart contract query and returns the result.
    ///
    /// # Parameters
    ///
    /// - `request`: A reference to the [`ScCallStep`] representing the smart contract query to be executed.
    ///
    /// # Type Parameters
    ///
    /// - `OutputManaged`: The managed type representing the expected output of the query.
    ///   It must implement [`TopDecodeMulti`], [`NativeConvertible`], [`Send`], and [`Sync`].
    ///
    /// # Returns
    ///
    /// A [`Result`] containing the native representation of the query result,
    /// or an [`ExecutorError`] if the query execution fails.
    async fn execute<OutputManaged>(&self, request: &ScCallStep) -> Result<OutputManaged::Native, ExecutorError>
        where
            OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync;
}

/// An implementation of `QueryExecutor` for `Arc<T>` where `T: QueryExecutor`.
///
/// This implementation allows shared access to an executor instance.
#[async_trait]
impl<T: QueryExecutor> QueryExecutor for Arc<T> {
    async fn execute<OutputManaged>(&self, request: &ScCallStep) -> Result<OutputManaged::Native, ExecutorError> where OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync {
        T::execute::<OutputManaged>(self, request).await
    }
}

/// An implementation of `QueryExecutor` for `Arc<Mutex<T>>` where `T: QueryExecutor`.
///
/// This implementation allows exclusive access to an executor instance, ensuring safe mutable access.
#[async_trait]
impl<T: QueryExecutor> QueryExecutor for Arc<Mutex<T>> {
    async fn execute<OutputManaged>(&self, request: &ScCallStep) -> Result<OutputManaged::Native, ExecutorError> where OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync {
        {
            let executor = self.lock().await;
            executor.execute::<OutputManaged>(request).await
        }
    }
}