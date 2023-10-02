use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use novax::Address;
use novax::errors::NovaXError;
use crate::errors::mocking::NovaXMockingError;

type Pairs = HashMap<String, String>;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct AddressKeysData {
    pub pairs: Pairs,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct AddressKeys {
    pub data: AddressKeysData,
    pub error: String,
    pub code: String,
}

impl AddressKeys {
    pub async fn from_gateway(gateway_url: &str, address: &Address) -> Result<AddressKeys, NovaXMockingError> {
        let bech32 = address.to_bech32_string().map_err(NovaXError::from)?;
        let url = format!("{gateway_url}/address/{bech32}/keys");

        let Ok(response) = reqwest::get(url).await else { return Err(NovaXMockingError::UnableToFetchAddressKeys.into()) };
        let Ok(response) = response.text().await else { return Err(NovaXMockingError::UnableToFetchAddressKeys.into()) };

        let Ok(result) = serde_json::from_str::<AddressKeys>(&response) else { return Err(NovaXMockingError::UnableToParseAddressKeys.into()) };

        Ok(result)
    }
}