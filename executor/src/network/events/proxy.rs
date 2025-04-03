use crate::error::executor::ExecutorError;
use async_trait::async_trait;
use elasticsearch::http::transport::Transport;
use elasticsearch::{Elasticsearch, SearchParts};
use serde_json::{json, Value};
use crate::network::events::models::events::ElasticSearchEvent;

#[async_trait]
pub trait ElasticSearchProxy: Send + Sync {
    fn new(elastic_search_url: String) -> Self;

    async fn execute(
        &self,
        contract_address: String,
        event_identifier: &str,
    ) -> Result<Vec<ElasticSearchEvent>, ExecutorError>;
}

pub struct ElasticSearchNodeProxy {
    pub gateway_url: String
}

#[async_trait]
impl ElasticSearchProxy for ElasticSearchNodeProxy {
    fn new(gateway_url: String) -> Self {
        Self {
            gateway_url,
        }
    }

    async fn execute(
        &self,
        contract_address: String,
        event_identifier: &str,
    ) -> Result<Vec<ElasticSearchEvent>, ExecutorError> {
        let transport = Transport::single_node(&self.gateway_url).unwrap();
        let client = Elasticsearch::new(transport);

        let event_identifier_hex = hex::encode(event_identifier);

        let query_body = json!({
            "from": 0,
            "sort": [
                {
                    "timestamp": "desc"
                }
            ],
            "query": {
                "bool": {
                    "filter": [
                        {
                            "match": {
                                "address": contract_address
                            }
                        },
                        {
                            "term": {
                                "topics": event_identifier_hex
                            }
                        }
                    ]
                }
            }
        });

        let response = client
            .search(SearchParts::Index(&["events"]))
            .pretty(true)
            .body(query_body)
            .send()
            .await
            .map_err(|err| -> ExecutorError { todo!() })?
            .json::<Value>()
            .await
            .map_err(|err| -> ExecutorError { todo!() })?;

        let Some(hits) = response["hits"]["hits"].as_array() else {
            todo!()
        };

        let mut logs: Vec<ElasticSearchEvent> = vec![];
        for hit in hits {
            let Some(source_raw) = hit.get("_source") else {
                todo!();
            };

            let Ok(decoded) = serde_json::from_value(source_raw.clone()) else {
                todo!()
            };

            logs.push(decoded);
        }

        Ok(logs)
    }
}