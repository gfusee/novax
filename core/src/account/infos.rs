use serde::{Deserialize, Serialize};
use novax_data::Address;
use crate::errors::{AccountError, NovaXError};

/// Struct to hold detailed data of an account from the blockchain.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfosAccountData {
    /// The address of the account as a string.
    pub address: String,
    /// The nonce value associated with the account.
    pub nonce: u64,
    /// The balance of the account as a string.
    pub balance: String,
    /// Optionally holds the code of the account, if any.
    /// This field is skipped during serialization if it holds a `None` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Optionally holds the owner address of the account, if any.
    /// This field is skipped during serialization if it holds a `None` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_address: Option<String>
}

/// Struct to hold the `AccountInfosAccountData` struct.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct AccountInfosData {
    /// Holds the account data.
    pub account: AccountInfosAccountData
}

/// Struct to encapsulate `AccountInfosData`.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct AccountInfos {
    /// Holds the data structure for account information.
    pub data: AccountInfosData
}

impl AccountInfos {
    /// Asynchronously retrieves account information from the specified gateway URL for the given address.
    ///
    /// # Arguments
    ///
    /// * `gateway_url` - The URL of the blockchain gateway to retrieve account information from.
    /// * `address` - The `Address` for which to retrieve account information.
    ///
    /// # Returns
    ///
    /// A `Result` with `AccountInfos` if the request and parsing are successful,
    /// or an `Err` wrapping a `NovaXError` if any error occurs during the process.
    pub async fn from_address(gateway_url: &str, address: &Address) -> Result<AccountInfos, NovaXError> {
        let bech32 = address.to_bech32_string()?;
        let url = format!("{}/address/{}", gateway_url, bech32);
        let response = reqwest::get(&url).await.map_err(|_| AccountError::CannotFetchAccountInfos)?;
        let response_text = response.text().await.map_err(|_| AccountError::CannotFetchAccountInfos)?;
        let result = serde_json::from_str(&response_text).map_err(|_| AccountError::CannotParseAccountInfos)?;
        Ok(result)
    }
}
