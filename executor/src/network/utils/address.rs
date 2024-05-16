use novax_data::Address;
use novax_request::gateway::client::GatewayClient;
use crate::error::gateway::GatewayError;
use crate::ExecutorError;
use crate::network::models::address::info::{AddressGatewayInfo, AddressGatewayResponse};

pub async fn get_address_info<Client: GatewayClient>(client: &Client, address: Address) -> Result<AddressGatewayInfo, ExecutorError> {
    let address_bech32 = address.to_bech32_string()?;

    let Ok((_, Some(text))) = client.with_appended_url(&format!("/address/{address_bech32}")).get().await else {
        return Err(GatewayError::CannotFetchAddressInfo { address: address_bech32 }.into())
    };

    let Ok(info) = serde_json::from_str::<AddressGatewayResponse>(&text) else {
        return Err(GatewayError::CannotParseAddressInfo { address: address_bech32 }.into())
    };

    let Some(data) = info.data else {
        return Err(GatewayError::NoDataForAddressInfo { address: address_bech32 }.into())
    };

    Ok(data)
}