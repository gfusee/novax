use std::str::FromStr;
use crate::error::executor::ExecutorError;
use crate::error::network_query_events::NetworkQueryEventsError;
use crate::network::events::models::events::ElasticSearchEvent;
use crate::utils::events::query_events_options::{EventQueryOptions, TimestampOption};
use async_trait::async_trait;
use elasticsearch::http::transport::Transport;
use elasticsearch::{Elasticsearch, SearchParts};
use serde_json::{json, Value};

#[async_trait]
pub trait ElasticSearchProxy: Send + Sync {
    fn new(elastic_search_url: String) -> Self;

    async fn execute(
        &self,
        contract_address: String,
        event_identifier: &str,
        options: Option<EventQueryOptions>,
        filter_terms_bytes: Vec<(Vec<u8>, u32)>
    ) -> Result<Vec<ElasticSearchEvent>, ExecutorError>;
}

pub struct ElasticSearchNodeProxy<Client>
where
    Client: ElasticSearchClient + Send + Sync,
{
    pub client: Client
}

#[async_trait]
pub trait ElasticSearchClient {
    fn new(elastic_url: String) -> Self;

    async fn search(&self, index: &str, query_body: Value) -> Result<Value, ExecutorError>;
}

#[async_trait]
impl ElasticSearchClient for Elasticsearch {
    fn new(elastic_url: String) -> Self {
        Elasticsearch::new(Transport::single_node(&elastic_url).unwrap())
    }

    async fn search(&self, index: &str, query_body: Value) -> Result<Value, ExecutorError> {
        let query_response = self
            .search(SearchParts::Index(&[index]))
            .pretty(true)
            .body(query_body)
            .send()
            .await
            .map_err(|err| -> ExecutorError { NetworkQueryEventsError::ErrorWhileSendingQuery { reason: err.to_string() }.into() })?
            .text()
            .await
            .map_err(|err| -> ExecutorError { NetworkQueryEventsError::ErrorWhileSendingQuery { reason: err.to_string() }.into() })?;

        Value::from_str(&query_response)
            .map_err(|_| -> ExecutorError { NetworkQueryEventsError::CannotDecodeQueryResponseToJSON { query_response }.into() })
    }
}

#[async_trait]
impl<Client> ElasticSearchProxy for ElasticSearchNodeProxy<Client>
where
    Client: ElasticSearchClient + Send + Sync,
{
    fn new(elastic_url: String) -> Self {
        Self {
            client: Client::new(elastic_url),
        }
    }

    async fn execute(
        &self,
        contract_address: String,
        event_identifier: &str,
        options: Option<EventQueryOptions>,
        filter_terms_bytes: Vec<(Vec<u8>, u32)>
    ) -> Result<Vec<ElasticSearchEvent>, ExecutorError> {
        let event_identifier_hex = hex::encode(event_identifier);

        let mut filters = vec![
            json!({
                "match": {
                    "address": contract_address
                }
            }),
            json!({
                "term": {
                    "topics": event_identifier_hex
                }
            })
        ];

        let mut filter_terms = filter_terms_bytes
            .iter()
            .map(|(term, _)| {
                let term_hex = hex::encode(term);

                json!({
                    "term": {
                        "topics": term_hex
                    }
                })
            })
            .collect();

        filters.append(&mut filter_terms);

        let mut query_body = json!({});

        if let Some(options) = options {
            if let Some(from) = options.from {
                query_body["from"] = json!(from);
            }

            if let Some(size) = options.size {
                query_body["size"] = json!(size);
            }

            if let Some(sort_options) = options.sort {
                let mut sort_values = vec![];

                if let Some(sort_timestamp) = sort_options.timestamp {
                    sort_values.push(json!({
                        "timestamp": sort_timestamp.as_elastic_search_term()
                    }));
                }

                query_body["sort"] = json!(sort_values);
            }

            if let Some(timestamp) = options.timestamp {
                let mut range_timestamp_value = json!({
                    "timestamp": {}
                });

                match timestamp {
                    TimestampOption::GreaterThanOrEqual(timestamp) => {
                        range_timestamp_value["timestamp"]["gte"] = Value::String(timestamp.to_string());
                    },
                    TimestampOption::LowerThanOrEqual(timestamp) => {
                        range_timestamp_value["timestamp"]["lte"] = Value::String(timestamp.to_string());
                    },
                    TimestampOption::Between(min, max) => {
                        range_timestamp_value["timestamp"]["gte"] = Value::String(min.to_string());
                        range_timestamp_value["timestamp"]["lte"] = Value::String(max.to_string());
                    }
                }

                filters.push(json!({
                    "range": range_timestamp_value
                }));
            }
        }

        query_body["query"] = json!({
            "bool": {
                "filter": filters
            }
        });

        let response = self.client.search("events", query_body).await?;

        let Some(hits) = response["hits"]["hits"].as_array() else {
            return Err(NetworkQueryEventsError::ResponseDoesntHaveHitsField { response: response.to_string() }.into());
        };

        let mut logs: Vec<ElasticSearchEvent> = vec![];
        for hit in hits {
            let Some(source_raw) = hit.get("_source") else {
                return Err(NetworkQueryEventsError::HitDoesntHaveSourceField { hit: hit.to_string() }.into());
            };

            let decoded = match serde_json::from_value(source_raw.clone()) {
                Ok(decoded) => decoded,
                Err(reason) => {
                    return Err(NetworkQueryEventsError::CannotDeserializeHitSource { hit: hit.to_string(), reason: reason.to_string() }.into());
                }
            };

            logs.push(decoded);
        }

        Ok(logs)
    }
}