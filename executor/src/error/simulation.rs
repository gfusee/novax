use serde::{Deserialize, Serialize};
use crate::ExecutorError;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum SimulationError {
    ErrorInTx { error: String, code: String }
}

impl From<SimulationError> for ExecutorError {
    fn from(value: SimulationError) -> Self {
        ExecutorError::Simulation(value)
    }
}