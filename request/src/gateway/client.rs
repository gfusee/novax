use async_trait::async_trait;
use reqwest::{Error, Response};
use serde::Serialize;

#[async_trait]
pub trait GatewayClient: Send + Sync {
    type Owned: GatewayClient;

    fn get_gateway_url(&self) -> &str;

    fn with_appended_url(&self, url: &str) -> Self::Owned;

    async fn get(&self) -> Result<Response, Error>;

    async fn post<Body>(&self, body: &Body) -> Result<Response, Error>
        where
            Body: Serialize + Send + Sync;
}