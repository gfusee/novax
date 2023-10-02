use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct AbiOutput {
    pub r#type: String,
}