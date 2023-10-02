use serde::Deserialize;
use crate::abi::input::AbiInput;
use crate::abi::output::AbiOutput;

pub type AbiInputs = Vec<AbiInput>;
pub type AbiOutputs = Vec<AbiOutput>;

#[derive(Deserialize, Clone, Debug)]
pub struct AbiEndpoint {
    pub name: String,
    pub mutability: AbiPossibleMutability,
    pub inputs: AbiInputs,
    pub outputs: AbiOutputs
}

#[derive(Deserialize, PartialEq, Clone, Debug)]
pub enum AbiPossibleMutability {
    #[serde(rename = "mutable")]
    Mutable,
    #[serde(rename = "readonly")]
    Readonly,
    Constructor
}