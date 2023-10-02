use std::borrow::Borrow;
use async_trait::async_trait;
use reqwest::{Client, Error, Response};
use serde::Serialize;
use crate::gateway::client::GatewayClient;

#[async_trait]
impl<T> GatewayClient for T
where
    T: Borrow<str> + Send + Sync + ?Sized
{
    type Owned = String;

    fn get_gateway_url(&self) -> &str {
        self.borrow()
    }

    fn with_appended_url(&self, url: &str) -> Self::Owned {
        format!("{}{url}", self.borrow())
    }

    async fn get(&self) -> Result<Response, Error> {
        Client::new()
            .get(self.borrow())
            .send()
            .await
    }

    async fn post<Body>(&self, body: &Body) -> Result<Response, Error> where Body: Serialize + Send + Sync {
        Client::new()
            .post(self.borrow())
            .json(body)
            .send()
            .await
    }
}