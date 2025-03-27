use crate::abi::event_input::AbiEventInput;
use serde::Deserialize;

pub type AbiEventInputs = Vec<AbiEventInput>;
#[derive(Deserialize, Clone, Debug)]
pub struct AbiEvent {
    pub identifier: String,
    pub inputs: AbiEventInputs,
}