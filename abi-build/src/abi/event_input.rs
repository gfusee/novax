use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct AbiEventInput {
    pub name: String,
    pub r#type: String,
    pub indexed: Option<bool>,
}