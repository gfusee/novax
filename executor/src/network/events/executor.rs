use crate::network::events::proxy::ElasticSearchProxy;
use crate::utils::events::query_events_options::EventQueryOptions;
use crate::utils::events::query_result::EventQueryResult;
use crate::{ElasticSearchNodeProxy, ExecutorError, IntoFilterTerms, QueryEventsExecutor, TopDecodeMulti};
use async_trait::async_trait;
use novax_data::{Address, NativeConvertible};
use std::marker::PhantomData;
use elasticsearch::Elasticsearch;
use crate::error::network_query_events::NetworkQueryEventsError;

pub type ElasticSearchNodeQueryExecutor = BaseElasticSearchNodeQueryExecutor<ElasticSearchNodeProxy<Elasticsearch>>;

#[derive(Clone, Debug)]
pub struct BaseElasticSearchNodeQueryExecutor<Proxy: ElasticSearchProxy> {
    /// The URL of the elastic search node.
    pub elastic_search_url: String,
    /// A phantom data field to keep the generic `Proxy` type.
    _data: PhantomData<Proxy>
}

impl<Proxy: ElasticSearchProxy> BaseElasticSearchNodeQueryExecutor<Proxy> {
    pub fn new(elastic_search_url: String) -> Self {
        Self {
            elastic_search_url,
            _data: PhantomData,
        }
    }
}

#[async_trait]
impl<Proxy: ElasticSearchProxy> QueryEventsExecutor for BaseElasticSearchNodeQueryExecutor<Proxy> {
    async fn execute<OutputManaged, FilterOptions>(
        &self,
        contract_address: &Address,
        event_identifier: &str,
        options: Option<EventQueryOptions>,
        filters: Option<FilterOptions>,
    ) -> Result<Vec<EventQueryResult<OutputManaged::Native>>, ExecutorError>
    where
        OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync,
        OutputManaged::Native: Send + Sync,
        FilterOptions: IntoFilterTerms + Send + Sync,
    {
        let proxy = Proxy::new(self.elastic_search_url.clone());

        let filter_terms = if let Some(filter_options) = filters {
            filter_options.into_filter_terms()
        } else {
            vec![]
        };

        let events = proxy
            .execute(
                contract_address.to_bech32_string()?,
                event_identifier,
                options,
                filter_terms.clone(),
            )
            .await?;

        let mut event_results: Vec<EventQueryResult<OutputManaged::Native>> = vec![];
        'outer: for event in events {
            let Some(event_identifier_raw) = event.topics.get(0) else {
                continue;
            };

            let event_identifier_bytes = match hex::decode(event_identifier_raw) {
                Ok(bytes) => bytes,
                Err(error) => {
                    return Err(NetworkQueryEventsError::CannotDecodeHexEventIdentifier { event_identifier: event_identifier_raw.to_string(), reason: error.to_string() }.into())
                }
            };

            let event_identifier_utf8 = match String::from_utf8(event_identifier_bytes.clone()) {
                Ok(string) => string,
                Err(error) => {
                    return Err(NetworkQueryEventsError::CannotGetUtf8EventIdentifierFromBytes { event_identifier_bytes, reason: error.to_string() }.into())
                }
            };

            for (term, position) in filter_terms.iter() {
                let Some(topic_raw) = event.topics.get(*position as usize) else {
                    continue 'outer;
                };

                let Ok(topic_bytes) = hex::decode(topic_raw) else {
                    continue 'outer;
                };

                if &topic_bytes != term {
                    continue 'outer;
                }
            }

            if event_identifier_utf8 != event_identifier {
                continue;
            };

            let mut data_to_decode = event.topics
                .get(1..)
                .map_or_else(Vec::new, |s| s.to_vec());
            data_to_decode.push(event.data.unwrap_or_default());

            let mut decoded_data_bytes = vec![];
            for data in &data_to_decode {
                let bytes = match hex::decode(&data) {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        return Err(NetworkQueryEventsError::CannotDecodeHexTopic { topic: data.to_string(), reason: error.to_string() }.into())
                    }
                };

                decoded_data_bytes.push(bytes);
            }

            let Ok(decoded_event) = OutputManaged::multi_decode(&mut decoded_data_bytes) else {
                return Err(NetworkQueryEventsError::CannotDeserializeTopicToContractType { topics: data_to_decode }.into())
            };

            event_results.push(
                EventQueryResult {
                    timestamp: event.timestamp,
                    event: decoded_event.to_native()
                }
            )
        }

        Ok(event_results)
    }
}