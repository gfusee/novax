use crate::error::executor::ExecutorError;
use crate::utils::events::decodable_event::DecodableEvent;
use crate::utils::events::query_events_options::EventQueryOptions;
use crate::utils::events::query_result::EventQueryResult;
use crate::IntoFilterTerms;
use async_trait::async_trait;
use novax_data::Address;
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait]
pub trait QueryEventsExecutor: Send + Sync {
    async fn execute<EventReturn, FilterOptions>(
        &self,
        contract_address: &Address,
        event_identifier: &str,
        options: Option<EventQueryOptions>,
        filters: Option<FilterOptions>
    ) -> Result<Vec<EventQueryResult<EventReturn>>, ExecutorError>
    where
        EventReturn: DecodableEvent + Send + Sync,
        FilterOptions: IntoFilterTerms + Send + Sync;
}

#[async_trait]
impl<T: QueryEventsExecutor> QueryEventsExecutor for Arc<T> {
    async fn execute<EventReturn, FilterOptions>(
        &self,
        contract_address: &Address,
        event_identifier: &str,
        options: Option<EventQueryOptions>,
        filters: Option<FilterOptions>
    ) -> Result<Vec<EventQueryResult<EventReturn>>, ExecutorError>
    where
        EventReturn: DecodableEvent + Send + Sync,
        FilterOptions: IntoFilterTerms + Send + Sync,
    {
        T::execute::<EventReturn, FilterOptions>(
            self,
            contract_address,
            event_identifier,
            options,
            filters
        ).await
    }
}

#[async_trait]
impl<T: QueryEventsExecutor> QueryEventsExecutor for Arc<Mutex<T>> {
    async fn execute<EventReturn, FilterOptions>(
        &self,
        contract_address: &Address,
        event_identifier: &str,
        options: Option<EventQueryOptions>,
        filters: Option<FilterOptions>
    ) -> Result<Vec<EventQueryResult<EventReturn>>, ExecutorError>
    where
        EventReturn: DecodableEvent + Send + Sync,
        FilterOptions: IntoFilterTerms + Send + Sync,
    {
        {
            let executor = self.lock().await;
            executor.execute::<EventReturn, FilterOptions>(
                contract_address,
                event_identifier,
                options,
                filters
            ).await
        }
    }
}