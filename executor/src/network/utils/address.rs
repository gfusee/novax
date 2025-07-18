use crate::error::gateway::GatewayError;
use crate::network::models::address::guardian::{AddressGatewayGuardianData, AddressGuardianDataGatewayResponse};
use crate::network::models::address::info::{AddressGatewayInfo, AddressGatewayResponse};
use crate::ExecutorError;
use novax_data::Address;
use novax_request::gateway::client::GatewayClient;

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

pub async fn get_address_guardian_data<Client: GatewayClient>(client: &Client, address: Address) -> Result<AddressGatewayGuardianData, ExecutorError> {
    let address_bech32 = address.to_bech32_string()?;

    let Ok((_, Some(text))) = client.with_appended_url(&format!("/address/{address_bech32}/guardian-data")).get().await else {
        return Err(GatewayError::CannotFetchAddressGuardianData { address: address_bech32 }.into())
    };
    
    let Ok(info) = serde_json::from_str::<AddressGuardianDataGatewayResponse>(&text) else {
        return Err(GatewayError::CannotParseAddressGuardianData { address: address_bech32 }.into())
    };

    let Some(data) = info.data else {
        return Err(GatewayError::NoDataForAddressGuardianData { address: address_bech32 }.into())
    };

    Ok(data)
}