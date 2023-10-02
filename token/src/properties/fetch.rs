use std::str::FromStr;
use async_trait::async_trait;
use multiversx_sc::types::ManagedBuffer;
use multiversx_sc_codec::multi_types::{MultiValue10, MultiValue2, MultiValue8};
use multiversx_sc_scenario::api::StaticApi;
use num_bigint::BigUint;
use serde_json::json;
use novax::errors::NovaXError;
use novax_data::Address;
use novax_data::NativeConvertible;
use novax_data::parse_query_return_string_data;
use novax_request::gateway::client::GatewayClient;
use crate::error::token::TokenError;
use crate::properties::model::TokenProperties;
use crate::properties::vm_response::TokenPropertiesResponse;

type RawProperties = MultiValue2<
    MultiValue10<
        ManagedBuffer<StaticApi>,
        ManagedBuffer<StaticApi>,
        Address,
        ManagedBuffer<StaticApi>,
        ManagedBuffer<StaticApi>,
        ManagedBuffer<StaticApi>,
        ManagedBuffer<StaticApi>,
        ManagedBuffer<StaticApi>,
        ManagedBuffer<StaticApi>,
        ManagedBuffer<StaticApi>,
    >,
    MultiValue8<
        ManagedBuffer<StaticApi>,
        ManagedBuffer<StaticApi>,
        ManagedBuffer<StaticApi>,
        ManagedBuffer<StaticApi>,
        ManagedBuffer<StaticApi>,
        ManagedBuffer<StaticApi>,
        ManagedBuffer<StaticApi>,
        ManagedBuffer<StaticApi>,
    >
>;

#[async_trait]
pub trait FetchTokenProperties {
    async fn fetch_token_properties<Client>(&self, gateway_client: &Client) -> Result<TokenProperties, TokenError>
    where
        Client: GatewayClient + ?Sized;
}

#[async_trait]
impl FetchTokenProperties for &str {
    async fn fetch_token_properties<Client>(&self, gateway_client: &Client) -> Result<TokenProperties, TokenError>
    where
        Client: GatewayClient + ?Sized
    {
        fetch_token_properties(gateway_client, self).await
    }
}

#[async_trait]
impl FetchTokenProperties for String {
    async fn fetch_token_properties<Client>(&self, gateway_client: &Client) -> Result<TokenProperties, TokenError> where Client: GatewayClient + ?Sized {
        self.as_str().fetch_token_properties(gateway_client).await
    }
}

async fn fetch_token_properties<Client>(gateway_client: &Client, token_identifier: &str) -> Result<TokenProperties, TokenError>
    where
        Client: GatewayClient + ?Sized
{
    let hex_identifier = hex::encode(token_identifier);

    let body = json!({
        "scAddress": "erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u",
        "funcName": "getTokenProperties",
        "args": [hex_identifier]
    });

    let response = gateway_client
        .with_appended_url("/vm-values/query")
        .post(
            &body
        )
        .await;

    let Ok(response) = response else {
        return Err(TokenError::UnknownErrorForToken { token_identifier: token_identifier.to_string() })
    };

    let Ok(text) = response.text().await else {
        return Err(TokenError::UnknownErrorForToken { token_identifier: token_identifier.to_string() })
    };

    let Ok(decoded) = serde_json::from_str::<TokenPropertiesResponse>(&text) else {
        return Err(TokenError::UnknownErrorForToken { token_identifier: token_identifier.to_string() })
    };

    let Some(return_data) = decoded.data.data.return_data else {
        if let Some(return_message) = decoded.data.data.return_message {
            if return_message == "no ticker with given name" {
                return Err(TokenError::TokenNotFound { token_identifier: token_identifier.to_string() })
            }
        }

        return Err(TokenError::UnknownErrorForToken { token_identifier: token_identifier.to_string() })
    };

    let result = parse_query_return_string_data::<RawProperties>(
        &return_data.iter().map(AsRef::as_ref).collect::<Vec<&str>>()
    )
        .map_err(NovaXError::from)?
        .to_native();


    let properties = TokenProperties {
        identifier: token_identifier.to_string(),
        name: result.0.0,
        r#type: result.0.1,
        owner: result.0.2.to_bech32_string().unwrap(),
        minted_value: BigUint::from_str(&result.0.3).unwrap(),
        burnt_value: BigUint::from_str(&result.0.4).unwrap(),
        decimals: string_to_u64(&result.0.5) as u8,
        is_paused: string_to_bool(&result.0.6),
        can_upgrade: string_to_bool(&result.0.7),
        can_mint: string_to_bool(&result.0.8),
        can_change_owner: string_to_bool(&result.0.9),
        can_pause: string_to_bool(&result.1.0),
        can_freeze: string_to_bool(&result.1.1),
        can_wipe: string_to_bool(&result.1.3),
        can_add_special_roles: string_to_bool(&result.1.4),
        can_transfer_nft_creation_role: string_to_bool(&result.1.5),
        nft_create_stopped: string_to_bool(&result.1.6),
        wiped_amount: string_to_bn(&result.1.7),
    };

    Ok(properties)
}

fn string_to_u64(value: &str) -> u64 {
    u64::from_str(value.split('-').collect::<Vec<&str>>().get(1).unwrap()).unwrap()
}

fn string_to_bn(value: &str) -> BigUint {
    BigUint::from_str(value.split('-').collect::<Vec<&str>>().get(1).unwrap()).unwrap()
}

fn string_to_bool(value: &str) -> bool {
    value.split('-').collect::<Vec<&str>>().get(1).unwrap() == &"true"
}

#[cfg(test)]
mod tests {
    use crate::error::token::TokenError;
    use crate::mock::request::MockClient;
    use crate::properties::fetch::{fetch_token_properties, FetchTokenProperties};
    use crate::properties::model::TokenProperties;

    #[tokio::test]
    async fn test_get_token_infos_with_valid_fungible() {
        let result = fetch_token_properties(&MockClient::new(),"WEGLD-d7c6bb").await.unwrap();

        let expected = TokenProperties {
            identifier: "WEGLD-d7c6bb".to_string(),
            name: "WrappedEGLD".to_string(),
            decimals: 18,
            is_paused: false,
            can_upgrade: true,
            can_mint: true,
            can_change_owner: true,
            can_pause: true,
            can_freeze: true,
            can_wipe: true,
            can_add_special_roles: true,
            can_transfer_nft_creation_role: false,
            nft_create_stopped: false,
            owner: "erd1x39tc3q3nn72ecjnmcz7x0qp09kp97t080x99dgyhx7zh95j0n4szskhlv".to_string(),
            minted_value: Default::default(),
            r#type: "FungibleESDT".to_string(),
            burnt_value: Default::default(),
            wiped_amount: Default::default(),
        };

        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_get_token_infos_with_valid_fungible_str_trait() {
        let result = "WEGLD-d7c6bb".fetch_token_properties(&MockClient::new()).await.unwrap();

        let expected = TokenProperties {
            identifier: "WEGLD-d7c6bb".to_string(),
            name: "WrappedEGLD".to_string(),
            decimals: 18,
            is_paused: false,
            can_upgrade: true,
            can_mint: true,
            can_change_owner: true,
            can_pause: true,
            can_freeze: true,
            can_wipe: true,
            can_add_special_roles: true,
            can_transfer_nft_creation_role: false,
            nft_create_stopped: false,
            owner: "erd1x39tc3q3nn72ecjnmcz7x0qp09kp97t080x99dgyhx7zh95j0n4szskhlv".to_string(),
            minted_value: Default::default(),
            r#type: "FungibleESDT".to_string(),
            burnt_value: Default::default(),
            wiped_amount: Default::default(),
        };

        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_get_token_infos_with_valid_fungible_string_trait() {
        let result = "WEGLD-d7c6bb".to_string().fetch_token_properties(&MockClient::new()).await.unwrap();

        let expected = TokenProperties {
            identifier: "WEGLD-d7c6bb".to_string(),
            name: "WrappedEGLD".to_string(),
            decimals: 18,
            is_paused: false,
            can_upgrade: true,
            can_mint: true,
            can_change_owner: true,
            can_pause: true,
            can_freeze: true,
            can_wipe: true,
            can_add_special_roles: true,
            can_transfer_nft_creation_role: false,
            nft_create_stopped: false,
            owner: "erd1x39tc3q3nn72ecjnmcz7x0qp09kp97t080x99dgyhx7zh95j0n4szskhlv".to_string(),
            minted_value: Default::default(),
            r#type: "FungibleESDT".to_string(),
            burnt_value: Default::default(),
            wiped_amount: Default::default(),
        };

        assert_eq!(result, expected);
    }


    #[tokio::test]
    async fn test_get_token_infos_with_valid_non_fungible() {
        let result = fetch_token_properties(&MockClient::new(),"AVASH-16f530").await.unwrap();

        let expected = TokenProperties {
            identifier: "AVASH-16f530".to_string(),
            name: "AVASH".to_string(),
            decimals: 18,
            is_paused: false,
            can_upgrade: false,
            can_mint: false,
            can_change_owner: false,
            can_pause: false,
            can_freeze: false,
            can_wipe: false,
            can_add_special_roles: true,
            can_transfer_nft_creation_role: false,
            nft_create_stopped: false,
            owner: "erd1qqqqqqqqqqqqqpgqaa2g2eev39xemwprezqg3j0mlktwurpzq33sd3j48g".to_string(),
            minted_value: Default::default(),
            r#type: "MetaESDT".to_string(),
            burnt_value: Default::default(),
            wiped_amount: Default::default(),
        };

        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_get_token_infos_with_invalid_identifier() {
        let result = fetch_token_properties(&MockClient::new(),"WEGLD-a").await.unwrap_err();

        let expected = TokenError::TokenNotFound { token_identifier: "WEGLD-a".to_string() };

        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_get_token_infos_with_unknown_fungible() {
        let result = fetch_token_properties(&MockClient::new(),"WEGLD-abcdef").await.unwrap_err();

        let expected = TokenError::TokenNotFound { token_identifier: "WEGLD-abcdef".to_string() };

        assert_eq!(result, expected);
    }
}