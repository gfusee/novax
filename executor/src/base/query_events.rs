use crate::error::executor::ExecutorError;
use crate::utils::events::query_result::EventQueryResult;
use async_trait::async_trait;
use multiversx_sc_scenario::multiversx_sc::codec::TopDecodeMulti;
use novax_data::{Address, NativeConvertible};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::IntoFilterTerms;

#[async_trait]
pub trait QueryEventsExecutor: Send + Sync {
    async fn execute<OutputManaged, FilterOptions>(
        &self,
        contract_address: &Address,
        event_identifier: &str,
        filter_options: Option<FilterOptions>
    ) -> Result<Vec<EventQueryResult<OutputManaged::Native>>, ExecutorError>
    where
        OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync,
        OutputManaged::Native: Serialize + DeserializeOwned + Send + Sync,
        FilterOptions: IntoFilterTerms + Send + Sync;
}

#[async_trait]
impl<T: QueryEventsExecutor> QueryEventsExecutor for Arc<T> {
    async fn execute<OutputManaged, FilterOptions>(
        &self,
        contract_address: &Address,
        event_identifier: &str,
        filter_options: Option<FilterOptions>
    ) -> Result<Vec<EventQueryResult<OutputManaged::Native>>, ExecutorError>
    where
        OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync,
        OutputManaged::Native: Serialize + DeserializeOwned + Send + Sync,
        FilterOptions: IntoFilterTerms + Send + Sync,
    {
        T::execute::<OutputManaged, FilterOptions>(
            self,
            contract_address,
            event_identifier,
            filter_options
        ).await
    }
}

#[async_trait]
impl<T: QueryEventsExecutor> QueryEventsExecutor for Arc<Mutex<T>> {
    async fn execute<OutputManaged, FilterOptions>(
        &self,
        contract_address: &Address,
        event_identifier: &str,
        filter_options: Option<FilterOptions>
    ) -> Result<Vec<EventQueryResult<OutputManaged::Native>>, ExecutorError>
    where
        OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync,
        OutputManaged::Native: Serialize + DeserializeOwned + Send + Sync,
        FilterOptions: IntoFilterTerms + Send + Sync,
    {
        {
            let executor = self.lock().await;
            executor.execute::<OutputManaged, FilterOptions>(
                contract_address,
                event_identifier,
                filter_options
            ).await
        }
    }
}