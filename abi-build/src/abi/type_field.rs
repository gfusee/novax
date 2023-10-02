use proc_macro2::Ident;
use quote::format_ident;
use serde::{Deserialize};
use crate::abi::managed_type::AbiManagedType;

#[derive(Deserialize, Clone, Debug)]
pub struct AbiTypeField {
    pub name: String,
    pub r#type: AbiManagedType
}

impl AbiTypeField {
    pub fn get_enum_field_name(&self) -> Option<Ident> {
        if self.name.parse::<u8>().is_ok() {
            None
        } else {
            Some(format_ident!("{}", self.name))
        }
    }
}