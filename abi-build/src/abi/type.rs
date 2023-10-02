use serde::{Deserialize};
use crate::abi::type_field::AbiTypeField;
use crate::abi::type_variant::AbiTypeVariant;

pub type AbiTypeFields = Vec<AbiTypeField>;
pub type AbiTypeVariants = Vec<AbiTypeVariant>;

#[derive(Deserialize, Clone, Debug)]
pub struct AbiType {
    pub r#type: AbiPossibleType,
    pub fields: Option<AbiTypeFields>,
    pub variants: Option<AbiTypeVariants>
}

#[derive(Deserialize, Clone, Debug)]
pub enum AbiPossibleType {
    #[serde(rename = "struct")]
    Struct,
    #[serde(rename = "enum")]
    Enum
}