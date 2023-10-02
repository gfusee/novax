use serde::Deserialize;
use crate::abi::r#type::AbiTypeFields;

#[derive(Deserialize, Clone, Debug)]
pub struct AbiTypeVariant {
    pub name: String,
    pub discriminant: u8,
    pub fields: Option<AbiTypeFields>
}