use crate::error::executor::ExecutorError;
use async_trait::async_trait;
use multiversx_sc_scenario::multiversx_sc::codec::TopDecodeMulti;
use novax_data::{Address, NativeConvertible};
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait]
pub trait QueryEventsExecutor: Send + Sync {
    async fn execute<OutputManaged>(
        &self,
        contract_address: &Address,
        event_identifier: &str,
    ) -> Result<Vec<OutputManaged::Native>, ExecutorError>
        where
            OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync;
}

#[async_trait]
impl<T: QueryEventsExecutor> QueryEventsExecutor for Arc<T> {
    async fn execute<OutputManaged>(
        &self,
        contract_address: &Address,
        event_identifier: &str,
    ) -> Result<Vec<OutputManaged::Native>, ExecutorError>
    where
        OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
    {
        T::execute::<OutputManaged>(
            self,
            contract_address,
            event_identifier
        ).await
    }
}

#[async_trait]
impl<T: QueryEventsExecutor> QueryEventsExecutor for Arc<Mutex<T>> {
    async fn execute<OutputManaged>(
        &self,
        contract_address: &Address,
        event_identifier: &str,
    ) -> Result<Vec<OutputManaged::Native>, ExecutorError>
    where
        OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
    {
        {
            let executor = self.lock().await;
            executor.execute::<OutputManaged>(
                contract_address,
                event_identifier
            ).await
        }
    }
}