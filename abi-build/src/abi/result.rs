use std::collections::HashMap;
use serde::{Deserialize};
use crate::abi::constructor::AbiConstructor;
use crate::abi::endpoint::AbiEndpoint;
use crate::abi::r#type::AbiType;

pub type AbiTypes = HashMap<String, AbiType>;
pub type AbiEndpoints = Vec<AbiEndpoint>;

#[derive(Deserialize, Clone, Debug)]
pub struct Abi {
    pub name: String,
    pub types: AbiTypes,
    pub endpoints: AbiEndpoints,
    pub constructor: AbiConstructor
}

impl Abi {
    pub(crate) fn get_mod_name(&self) -> String {
        self.name.to_lowercase().replace(' ', "_")
    }

    pub(crate) fn get_contract_name(&self) -> String {
        match &self.name {
            s if s.ends_with("Contract") => self.name.clone(),
            _ =>  self.name.clone() + "Contract"
        }
    }

    pub(crate) fn get_proxy_name(&self) -> String {
        self.get_contract_name() + "Proxy"
    }

    pub(crate) fn get_query_name(&self) -> String {
        self.get_contract_name() + "Query"
    }

    pub(crate) fn get_call_name(&self) -> String {
        self.get_contract_name() + "Call"
    }
}