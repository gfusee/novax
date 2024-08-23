use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use async_trait::async_trait;
use base64::Engine;
use novax::CodeMetadata;
use num_bigint::BigUint;
use novax_data::Address;
use serde::{Deserialize, Serialize};
use novax::caching::CachingStrategy;
use novax::errors::NovaXError;
use novax_request::gateway::client::GatewayClient;
use crate::error::account::AccountError;
use crate::utils::data::{code_metadata_deserialize, code_metadata_serialize};

#[derive(Serialize, Deserialize, Default)]
struct GatewayAccountInfo {
    pub address: String,
    pub nonce: u64,
    pub balance: String,
    pub username: String,
    pub code: String,
    #[serde(rename = "codeHash")]
    pub code_hash: Option<String>,
    #[serde(rename = "rootHash")]
    pub root_hash: Option<String>,
    #[serde(rename = "codeMetadata")]
    pub code_metadata: Option<String>,
    #[serde(rename = "developerReward")]
    pub developer_reward: String,
    #[serde(rename = "ownerAddress")]
    pub owner_address: String,

}

#[derive(Serialize, Deserialize, Default)]
struct GatewayBlockInfo {
    pub nonce: u64,
    pub hash: String,
    #[serde(rename = "rootHash")]
    pub root_hash: String

}

#[derive(Serialize, Deserialize, Default)]
struct GatewayAccountInfoData {
    pub account: GatewayAccountInfo,
    #[serde(rename = "blockInfo")]
    pub block_info: GatewayBlockInfo

}

#[derive(Serialize, Deserialize, Default)]
struct GatewayAccount {
    data: GatewayAccountInfoData
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
pub struct AccountInfo {
    pub address: Address,
    pub nonce: u64,
    pub balance: BigUint,
    pub username: String,
    pub code: Option<String>,
    #[serde(rename = "codeHash")]
    pub code_hash: Option<String>,
    #[serde(rename = "rootHash")]
    pub root_hash: Option<String>,
    #[serde(serialize_with = "code_metadata_serialize")]
    #[serde(deserialize_with = "code_metadata_deserialize")]
    #[serde(rename = "codeMetadata")]
    pub code_metadata: Option<CodeMetadata>,
    #[serde(rename = "developerReward")]
    pub developer_reward: BigUint,
    #[serde(rename = "ownerAddress")]
    pub owner_address: Option<Address>,

}

#[async_trait]
pub trait FetchAccount {
    async fn fetch_account_info<Client, Caching: CachingStrategy>(&self, gateway_client: &Client, caching: &Caching) -> Result<AccountInfo, AccountError>
    where
        Client: GatewayClient + ?Sized,
        Caching: CachingStrategy;
}

#[async_trait]
impl FetchAccount for Address {
    async fn fetch_account_info<Client, Caching: CachingStrategy>(&self, gateway_client: &Client, caching: &Caching) -> Result<AccountInfo, AccountError>
    where
        Client: GatewayClient + ?Sized,
        Caching: CachingStrategy {
            fetch_account_info_for_address(gateway_client, self, caching).await
        }
}

async fn fetch_account_info_for_address<Client, Caching>(gateway_client: &Client, address: &Address, caching: &Caching) -> Result<AccountInfo, AccountError>
    where
        Client: GatewayClient + ?Sized,
        Caching: CachingStrategy
{
    let bech32_address = address.to_bech32_string().map_err(NovaXError::from)?;
    let client = gateway_client.with_appended_url(&format!("/address/{}", bech32_address));
    let key = format!("fetch_account_info_for_address_{}_{bech32_address}", client.get_gateway_url());
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);

    caching.get_or_set_cache(
        hasher.finish(),
        async {
            let Ok((_, Some(text))) = client.get().await else { return Err(AccountError::UnknownErrorWhileGettingInfosOfAccount { address: address.to_string() }) };
            let Ok(decoded) = serde_json::from_str::<GatewayAccount>(&text) else {
                return Err(AccountError::CannotParseAccountInfo { address: address.to_string() })
            };

            let raw_info = decoded.data.account;
            let Ok(balance) = BigUint::from_str(&raw_info.balance) else {
                return Err(AccountError::CannotParseAccountBalance { address: bech32_address, balance: raw_info.balance})
            };

            let Ok(developer_reward) = BigUint::from_str(&raw_info.developer_reward) else {
                return Err(AccountError::CannotParseAccountDeveloperReward { address: bech32_address, reward: raw_info.developer_reward})
            };

            let owner_address = if raw_info.owner_address.len() == 0 {
               None
            } else {
                let Ok(address) = Address::from_bech32_string(&raw_info.owner_address) else {
                    return Err(AccountError::CannotParseAccountOwnerAddress { address: bech32_address, owner: raw_info.owner_address})
                };
                Some(address)
            };

            let code_metadata = if let Some(raw_code_metadata) = raw_info.code_metadata {
                Some(decode_code_metadata(raw_code_metadata)?)
            } else {
                None
            };

            let code = if raw_info.code.len() == 0 {
                None
            } else {
                Some(raw_info.code)
            };

            Ok(AccountInfo {
                address: address.clone(),
                nonce: raw_info.nonce,
                balance,
                username: raw_info.username,
                code,
                code_hash: raw_info.code_hash,
                root_hash: raw_info.root_hash,
                code_metadata,
                developer_reward,
                owner_address,
            })
        }
    ).await
}

fn decode_code_metadata(encoded: String) -> Result<CodeMetadata, AccountError> {
    let decoded_bytes = base64::engine::general_purpose::STANDARD.decode(encoded.clone()).or(Err(AccountError::CannotDecodeCodeMetadata { metadata: encoded.clone() }))?;
    if decoded_bytes.len() != 2 {
        return Err(AccountError::CannotDecodeCodeMetadata { metadata: encoded });
    }

    let byte_array: [u8; 2] = decoded_bytes.as_slice().try_into().or(Err(AccountError::CannotDecodeCodeMetadata { metadata: encoded }))?;
    Ok(CodeMetadata::from(byte_array))
}

#[cfg(test)]
mod tests {
    use novax::CodeMetadata;
    use num_bigint::BigUint;
    use novax::caching::CachingNone;
    use novax_data::Address;
    use crate::account::info::{decode_code_metadata, fetch_account_info_for_address, AccountInfo};
    use crate::account::info::AccountError::CannotParseAccountInfo;
    use crate::mock::request::MockClient;

    #[test]
    pub fn test_all_code_metadata_decoding() {
        // Given 
        let code_metadata_string = "BQY=".to_string();
        // When
        let code_metadata = decode_code_metadata(code_metadata_string).expect("code meta data should be decodable");
        // Then
        assert_eq!(code_metadata, CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE | CodeMetadata::PAYABLE | CodeMetadata::PAYABLE_BY_SC);
    }

    #[test]
    pub fn test_no_code_metadata_decoding() {
        // Given 
        let code_metadata_string = "AAA=".to_string();
        // When
        let code_metadata = decode_code_metadata(code_metadata_string).expect("code meta data should be decodable");
        // Then
        assert_eq!(code_metadata, CodeMetadata::DEFAULT);
    }

    #[test]
    pub fn test_some_code_metadata_decoding() {
        // Given 
        let code_metadata_string = "BQQ=".to_string();
        // When
        let code_metadata = decode_code_metadata(code_metadata_string).expect("code meta data should be decodable");
        // Then
        assert_eq!(code_metadata, CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE | CodeMetadata::PAYABLE_BY_SC);
    }

    #[tokio::test]
    pub async fn test_with_valid_sc_address() {
        let result = fetch_account_info_for_address(&MockClient::new(), &"erd1qqqqqqqqqqqqqpgqr7een4m5z44frr3k35yjdjcrfe6703cwdl3s3wkddz".into(), &CachingNone).await.unwrap();

        assert_eq!(result, AccountInfo {
            address: "erd1qqqqqqqqqqqqqpgqr7een4m5z44frr3k35yjdjcrfe6703cwdl3s3wkddz".into(),
            nonce: 0,
            balance: BigUint::from(0u64),
            username: "".to_string(),
            code: Some("fakecodestring".to_string()),
            code_hash: Some("gVgRRf6HhmTGlxziasAFoCgBlP7/DH0i9IhTbj7lsxA=".to_string()),
            root_hash: Some("A3RZ7aYh4NzkunNL+fu09ggnItEeC7SuPWJDfIHmAcI=".to_string()),
            code_metadata: Some(CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE),
            developer_reward: BigUint::from(2288888045322000000u64),
            owner_address: Some(Address::from_bech32_string("erd1kj7l40rmklhp06treukh8c2merl2h78v2939wyxwc5000t25dl3s85klfd").unwrap())
        });

    }

    #[tokio::test]
    pub async fn test_with_valid_user_address() {
        let result = fetch_account_info_for_address(&MockClient::new(), &"erd1kj7l40rmklhp06treukh8c2merl2h78v2939wyxwc5000t25dl3s85klfd".into(), &CachingNone).await.unwrap();


        assert_eq!(result, AccountInfo {
            address: "erd1kj7l40rmklhp06treukh8c2merl2h78v2939wyxwc5000t25dl3s85klfd".into(),
            nonce: 6,
            balance: BigUint::from(412198271210000000u64),
            username: "".to_string(),
            code: None,
            code_hash: None,
            root_hash: Some("Juj3aJQOKv4DzZG3XOueG934NL7pq/7bmiVnR4zzXAo=".to_string()),
            code_metadata: None,
            developer_reward: BigUint::from(0u64),
            owner_address: None
        });
    }

    #[tokio::test]
    pub async fn test_with_invalid_address() {
        let address = Address::from_bytes([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let result = fetch_account_info_for_address(&MockClient::new(), &address, &CachingNone).await;

        assert_eq!(result, Err(CannotParseAccountInfo { address: "erd1qyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqsl6e0p7".to_string() }))

    }

    #[tokio::test]
    pub async fn test_with_non_existant_address() {
        let result = fetch_account_info_for_address(&MockClient::new(), &"erd16k7f023jt0a6wgnlwv4c2lz42p7t64xlsvk8a3d6vu6l5cl4htmseymu7y".into(), &CachingNone).await.unwrap();

        assert_eq!(result, AccountInfo {
            address: "erd16k7f023jt0a6wgnlwv4c2lz42p7t64xlsvk8a3d6vu6l5cl4htmseymu7y".into(),
            nonce: 0,
            balance: BigUint::from(0u64),
            username: "".to_string(),
            code: None,
            code_hash: None,
            root_hash: None,
            code_metadata: None,
            developer_reward: BigUint::from(0u64),
            owner_address: None
        });


    }
}