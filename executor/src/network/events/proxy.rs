use async_trait::async_trait;

use crate::error::executor::ExecutorError;

#[async_trait]
pub trait ElasticSearchProxy: Send + Sync {
    fn new(elastic_search_url: String) -> Self;

    async fn execute(
        &self,
    ) -> Result<(), ExecutorError>;
}

pub struct ElasticSearchServerProxy {
    pub gateway_url: String
}

#[async_trait]
impl ElasticSearchProxy for ElasticSearchServerProxy {
    fn new(gateway_url: String) -> Self {
        Self {
            gateway_url,
        }
    }

    async fn execute(&self) -> Result<(), ExecutorError> {
        todo!()
    }
}