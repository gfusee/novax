use std::marker::PhantomData;
use async_trait::async_trait;
use base64::Engine;
use novax_data::{Address, NativeConvertible};
use crate::network::events::proxy::ElasticSearchProxy;
use crate::{ElasticSearchNodeProxy, ExecutorError, QueryEventsExecutor, TopDecodeMulti};
use crate::results::decode_topic;
use crate::utils::events::query_result::EventQueryResult;

pub type ElasticSearchNodeQueryExecutor = BaseElasticSearchNodeQueryExecutor<ElasticSearchNodeProxy>;

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
    async fn execute<OutputManaged>(
        &self,
        contract_address: &Address,
        event_identifier: &str
    ) -> Result<Vec<EventQueryResult<OutputManaged::Native>>, ExecutorError>
    where
        OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync,
        OutputManaged::Native: Send + Sync
    {
        let proxy = Proxy::new(self.elastic_search_url.clone());

        let events = proxy
            .execute(
                contract_address.to_bech32_string()?,
                event_identifier,
            )
            .await?;

        let mut event_results: Vec<EventQueryResult<OutputManaged::Native>> = vec![];
        for event in events {
            let Some(event_identifier_raw) = event.topics.get(0) else {
                continue;
            };

            let Ok(event_identifier_bytes) = hex::decode(event_identifier_raw) else {
                todo!()
            };

            let Ok(event_identifier_utf8) = String::from_utf8(event_identifier_bytes) else {
                todo!()
            };

            if event_identifier_utf8 != event_identifier {
                continue;
            };

            let mut data_to_decode = event.topics
                .get(1..)
                .map_or_else(Vec::new, |s| s.to_vec());
            data_to_decode.push(event.data);

            let mut decoded_data_bytes = vec![];
            for data in data_to_decode {
                let Ok(bytes) = hex::decode(data) else {
                    todo!()
                };

                decoded_data_bytes.push(bytes);
            }

            let Ok(decoded_event) = OutputManaged::multi_decode(&mut decoded_data_bytes) else {
                todo!()
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