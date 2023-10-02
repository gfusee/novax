use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct AbiInput {
    pub name: String,
    pub r#type: String
}