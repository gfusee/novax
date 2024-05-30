use std::borrow::Borrow;

use async_trait::async_trait;
use http::StatusCode;
use reqwest::Client;
use serde::Serialize;
use crate::error::client::ClientError;

use crate::error::request::RequestError;
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

    async fn get(&self) -> Result<(StatusCode, Option<String>), RequestError> {
        let response = Client::new()
            .get(self.borrow())
            .send()
            .await
            .map_err(|_| ClientError::UnknownError)?;

        let status = response.status();

        let text = if status.is_success() {
            let text = response.text().await.unwrap(); // TODO

            Some(text)
        } else {
            None
        };

        Ok((status, text))
    }

    async fn post<Body>(&self, body: &Body) -> Result<(StatusCode, Option<String>), RequestError> where Body: Serialize + Send + Sync {
        let response = Client::new()
            .post(self.borrow())
            .json(body)
            .send()
            .await
            .map_err(|_| ClientError::UnknownError)?;

        let status = response.status();

        let text = if status.is_success() {
            let text = response.text().await.unwrap(); // TODO

            Some(text)
        } else {
            None
        };

        Ok((status, text))
    }
}