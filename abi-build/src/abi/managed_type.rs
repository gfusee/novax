use proc_macro2::{TokenStream};
use serde::{Deserialize, Deserializer};
use crate::abi::result::AbiTypes;
use crate::errors::build_error::BuildError;
use crate::utils::parse_abi_type_name_to_managed_ident::parse_abi_type_name_to_managed_ident;

#[derive(Clone, Debug)]
pub struct AbiManagedType(pub String);

impl AbiManagedType {
    pub fn get_managed_type_ident(&self, api_generic: &TokenStream, abi_types: &AbiTypes) -> Result<TokenStream, BuildError> {
        parse_abi_type_name_to_managed_ident(
            self.0.as_str(),
            abi_types,
            api_generic
        )
    }
}

impl<'de> Deserialize<'de> for AbiManagedType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        Ok(AbiManagedType(String::deserialize(deserializer)?))
    }
}