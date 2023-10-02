use serde::Deserialize;
use crate::abi::endpoint::{AbiEndpoint, AbiPossibleMutability};
use crate::abi::input::AbiInput;
use crate::abi::output::AbiOutput;

pub type AbiInputs = Vec<AbiInput>;
pub type AbiOutputs = Vec<AbiOutput>;

#[derive(Deserialize, Clone, Debug)]
pub struct AbiConstructor {
    pub inputs: AbiInputs,
    pub outputs: AbiOutputs
}

impl AbiConstructor {
    pub fn into_endpoint(self) -> AbiEndpoint {
        AbiEndpoint {
            name: "init".to_string(),
            mutability: AbiPossibleMutability::Constructor,
            inputs: self.inputs,
            outputs: self.outputs,
        }
    }
}