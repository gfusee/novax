use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use async_trait::async_trait;
use num_bigint::BigUint;
use novax_data::Address;
use serde::{Deserialize, Serialize};
use novax::caching::CachingStrategy;
use novax::errors::NovaXError;
use novax_request::gateway::client::GatewayClient;
use crate::error::account::AccountError;

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
    pub root_hash: String,
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

#[derive(Serialize, Deserialize, Default)]
pub struct AccountInfo {
    pub nonce: u64,
    pub balance: BigUint,
    pub username: String,
    pub code: String,
    #[serde(rename = "codeHash")]
    pub code_hash: Option<String>,
    #[serde(rename = "rootHash")]
    pub root_hash: String,
    #[serde(rename = "codeMetadata")]
    pub code_metadata: Option<String>, // TODO: parse code metadata
    #[serde(rename = "developerReward")]
    pub developer_reward: BigUint,
    #[serde(rename = "ownerAddress")]
    pub owner_address: Address,

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
                Address::default()
            } else {
                let Ok(address) = Address::from_bech32_string(&raw_info.owner_address) else {
                    return Err(AccountError::CannotParseAccountOwnerAddress { address: bech32_address, owner: raw_info.owner_address})
                };
                address
            };

            Ok(AccountInfo {
                nonce: raw_info.nonce,
                balance,
                username: raw_info.username,
                code: raw_info.code,
                code_hash: raw_info.code_hash,
                root_hash: raw_info.root_hash,
                code_metadata: raw_info.code_metadata, // TODO: parse code metadata
                developer_reward: developer_reward,
                owner_address,
            })
        }
    ).await
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;
    use novax::caching::CachingNone;
    use novax_data::Address;
    use crate::account::info::fetch_account_info_for_address;
    use crate::mock::request::MockClient;

    #[tokio::test]
    pub async fn test_with_valid_sc_address() {
        let result = fetch_account_info_for_address(&MockClient::new(), &"erd1qqqqqqqqqqqqqpgqr7een4m5z44frr3k35yjdjcrfe6703cwdl3s3wkddz".into(), &CachingNone).await.unwrap();

        assert_eq!(result.nonce, 0);
        assert_eq!(result.balance, BigUint::from(0u64));
        assert_eq!(result.username, "".to_string());
        assert_eq!(result.code,"fakecodestring".to_string(),);
        assert_eq!(result.code_hash, Some("gVgRRf6HhmTGlxziasAFoCgBlP7/DH0i9IhTbj7lsxA=".to_string()));
        assert_eq!(result.root_hash, "A3RZ7aYh4NzkunNL+fu09ggnItEeC7SuPWJDfIHmAcI=".to_string());
        assert_eq!(result.code_metadata, Some("BQA=".to_string()));
        assert_eq!(result.developer_reward, BigUint::from(2288888045322000000u64));
        assert_eq!(result.owner_address, Address::from_bech32_string("erd1kj7l40rmklhp06treukh8c2merl2h78v2939wyxwc5000t25dl3s85klfd").unwrap());
    }

    #[tokio::test]
    pub async fn test_with_valid_user_address() {
        let result = fetch_account_info_for_address(&MockClient::new(), &"erd1kj7l40rmklhp06treukh8c2merl2h78v2939wyxwc5000t25dl3s85klfd".into(), &CachingNone).await.unwrap();

        assert_eq!(result.nonce, 6);
        assert_eq!(result.balance, BigUint::from(412198271210000000u64));
        assert_eq!(result.username, "".to_string());
        assert_eq!(result.code,"".to_string(),);
        assert_eq!(result.code_hash, None);
        assert_eq!(result.root_hash, "Juj3aJQOKv4DzZG3XOueG934NL7pq/7bmiVnR4zzXAo=".to_string());
        assert_eq!(result.code_metadata, None);
        assert_eq!(result.developer_reward, BigUint::from(0u64));
        assert_eq!(result.owner_address, Address::default());
    }
}