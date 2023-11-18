use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use async_trait::async_trait;
use base64::Engine;
use num_bigint::BigUint;
use novax_data::Address;
use serde::{Deserialize, Serialize};
use novax::caching::CachingStrategy;
use novax::errors::NovaXError;
use novax_request::gateway::client::GatewayClient;
use crate::error::token::TokenError;

#[derive(Serialize, Deserialize, Default)]
struct GatewayAllEsdtsForAddressEsdtInfos {
    #[serde(rename = "tokenIdentifier")]
    pub token_identifier: String,
    pub nonce: Option<u64>,
    pub balance: String,
    pub attributes: Option<String>,
}

type GatewayAllEsdtsForAddressEsdts = HashMap<String, GatewayAllEsdtsForAddressEsdtInfos>;

#[derive(Serialize, Deserialize)]
struct GatewayAllEsdtForAddressData {
    pub esdts: GatewayAllEsdtsForAddressEsdts,
}

#[derive(Serialize, Deserialize)]
struct GatewayAllEsdtForAddress {
    pub data: GatewayAllEsdtForAddressData,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct TokenInfos {
    pub token_identifier: String,
    pub nonce: u64,
    pub balance: BigUint,
    pub attributes: Option<Vec<u8>>,
}

#[async_trait]
pub trait FetchAllTokens {
    async fn fetch_all_tokens<Client, Caching: CachingStrategy>(&self, gateway_client: &Client, caching: &Caching) -> Result<Vec<TokenInfos>, TokenError>
    where
        Client: GatewayClient + ?Sized,
        Caching: CachingStrategy;
}

#[async_trait]
impl FetchAllTokens for Address {
    async fn fetch_all_tokens<Client, Caching: CachingStrategy>(&self, gateway_client: &Client, caching: &Caching) -> Result<Vec<TokenInfos>, TokenError>
    where
        Client: GatewayClient + ?Sized,
        Caching: CachingStrategy
    {
        fetch_all_tokens_for_address(gateway_client, self, caching).await
    }
}

async fn fetch_all_tokens_for_address<Client, Caching>(gateway_client: &Client, address: &Address, caching: &Caching) -> Result<Vec<TokenInfos>, TokenError>
    where
        Client: GatewayClient + ?Sized,
        Caching: CachingStrategy
{
    let bech32_address = address.to_bech32_string().map_err(NovaXError::from)?;
    let client = gateway_client.with_appended_url(&format!("/address/{}/esdt", bech32_address));
    let key = format!("fetch_all_tokens_for_address_{}_{bech32_address}", client.get_gateway_url());
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);

    caching.get_or_set_cache(
        hasher.finish(),
        async {
            let Ok(response) = client.get().await else { return Err(TokenError::UnknownErrorWhileGettingEsdtInfosOfAddress { address: address.to_string() }) };
            let Ok(response) = response.text().await else { return Err(TokenError::UnknownErrorWhileGettingEsdtInfosOfAddress { address: address.to_string() }) };
            let Ok(decoded) = serde_json::from_str::<GatewayAllEsdtForAddress>(&response) else {
                return Err(TokenError::CannotParseEsdtBalances { address: address.to_string() })
            };

            let mut results = vec![];

            for (_, raw_infos) in decoded.data.esdts {
                let Ok(balance) = BigUint::from_str(&raw_infos.balance) else {
                    return Err(TokenError::UnableToParseBigUintBalanceForTokenAndAddress {
                        token_identifier: raw_infos.token_identifier,
                        address: bech32_address,
                        balance: raw_infos.balance,
                    })
                };

                let decoded_attributes = if let Some(raw_attributes) = raw_infos.attributes {
                    let Ok(decoded_attributes) = base64::engine::general_purpose::STANDARD.decode(
                        raw_attributes
                    ) else {
                        return Err(TokenError::CannotDecodeBase64Attributes {
                            token_identifier: raw_infos.token_identifier,
                            nonce: raw_infos.nonce.unwrap_or_default()
                        })
                    };

                    Some(decoded_attributes)
                } else {
                    None
                };

                let infos = TokenInfos {
                    token_identifier: raw_infos.token_identifier,
                    nonce: raw_infos.nonce.unwrap_or(0),
                    balance,
                    attributes: decoded_attributes,
                };

                results.push(infos);
            }

            Ok(results)
        }
    ).await
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use base64::Engine;
    use num_bigint::BigUint;
    use novax::caching::CachingNone;
    use novax_data::Address;
    use crate::account::balance::{fetch_all_tokens_for_address, FetchAllTokens, TokenInfos};
    use crate::mock::request::MockClient;

    #[tokio::test]
    pub async fn test_with_valid_address() {
        let mut result = fetch_all_tokens_for_address(&MockClient::new(), &"erd1n7ed3f6rkqvwkpfevulvhyl4hskx2vqyleed5lqfq9jp2csfw8esg88f5g".into(), &CachingNone).await.unwrap();
        result.sort_by(|a, b| a.token_identifier.partial_cmp(&b.token_identifier).unwrap());

        let expected_len = 60;
        let expected_fungible = TokenInfos {
            token_identifier: "WEGLD-d7c6bb".to_string(),
            nonce: 0,
            balance: BigUint::from_str("71179029947004300508").unwrap(),
            attributes: None,
        };

        let expected_attributes_bytes = base64::engine::general_purpose::STANDARD.decode("AAAABBQU4X0AAAAE7ydxXJ+y2KdDsBjrBTlnPsuT9bwsZTAE/nLafAkBZBViCXHzAAAACA3gtrOnZAAAAAAACA3gtrOnZAAAAAAAAA==").unwrap();
        let expected_non_fungible = TokenInfos {
            token_identifier: "FARM-c4c5ef-1f52".to_string(),
            nonce: 8018,
            balance: BigUint::from_str("1000000000000000000").unwrap(),
            attributes: Some(expected_attributes_bytes),
        };

        assert_eq!(result.len(), expected_len);
        assert_eq!(result[57], expected_fungible);
        assert_eq!(result[36], expected_non_fungible);
    }

    #[tokio::test]
    pub async fn test_with_valid_address_on_struct() {
        let address: Address = "erd1n7ed3f6rkqvwkpfevulvhyl4hskx2vqyleed5lqfq9jp2csfw8esg88f5g".into();
        let mut result = address.fetch_all_tokens(&MockClient::new(), &CachingNone).await.unwrap();
        result.sort_by(|a, b| a.token_identifier.partial_cmp(&b.token_identifier).unwrap());

        let expected_len = 60;
        let expected_fungible = TokenInfos {
            token_identifier: "WEGLD-d7c6bb".to_string(),
            nonce: 0,
            balance: BigUint::from_str("71179029947004300508").unwrap(),
            attributes: None,
        };

        let expected_attributes_bytes = base64::engine::general_purpose::STANDARD.decode("AAAABBQU4X0AAAAE7ydxXJ+y2KdDsBjrBTlnPsuT9bwsZTAE/nLafAkBZBViCXHzAAAACA3gtrOnZAAAAAAACA3gtrOnZAAAAAAAAA==").unwrap();
        let expected_non_fungible = TokenInfos {
            token_identifier: "FARM-c4c5ef-1f52".to_string(),
            nonce: 8018,
            balance: BigUint::from_str("1000000000000000000").unwrap(),
            attributes: Some(expected_attributes_bytes),
        };

        assert_eq!(result.len(), expected_len);
        assert_eq!(result[57], expected_fungible);
        assert_eq!(result[36], expected_non_fungible);
    }
}