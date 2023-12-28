use novax_request::gateway::client::GatewayClient;
use crate::error::gateway::GatewayError;
use crate::ExecutorError;
use crate::network::models::network::config::{NetworkGatewayConfig, NetworkGatewayConfigResponse};

pub async fn get_network_config<Client: GatewayClient>(client: &Client) -> Result<NetworkGatewayConfig, ExecutorError> {
    let Ok(response) = client.with_appended_url("/network/config").get().await else {
        return Err(GatewayError::CannotFetchNetworkConfig.into())
    };

    let Ok(text) = response.text().await else {
        return Err(GatewayError::CannotFetchNetworkConfig.into())
    };

    let Ok(info) = serde_json::from_str::<NetworkGatewayConfigResponse>(&text) else {
        return Err(GatewayError::CannotParseNetworkConfig.into())
    };

    let Some(data) = info.data else {
        return Err(GatewayError::CannotParseNetworkConfig.into())
    };

    Ok(data)
}