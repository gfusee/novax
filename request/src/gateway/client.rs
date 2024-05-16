use async_trait::async_trait;
use http::StatusCode;
use serde::Serialize;
use crate::error::request::RequestError;

#[async_trait]
pub trait GatewayClient: Send + Sync {
    type Owned: GatewayClient;

    fn get_gateway_url(&self) -> &str;

    fn with_appended_url(&self, url: &str) -> Self::Owned;

    async fn get(&self) -> Result<(StatusCode, Option<String>), RequestError>;

    async fn post<Body>(&self, body: &Body) -> Result<(StatusCode, Option<String>), RequestError>
        where
            Body: Serialize + Send + Sync;
}