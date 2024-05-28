use serde::{Deserialize, Serialize};
use crate::ExecutorError;

/// An enumeration representing errors that can occur during the simulation of transactions or contract executions.
/// These errors are specific to the logic or processing of the transaction/contract, rather than network communication or data formatting issues.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum SimulationError {
    /// Error encountered within the transaction or contract execution during simulation.
    ErrorInTx {
        /// A string describing the error that occurred during the transaction or contract execution.
        /// Provides a human-readable explanation of the issue.
        error: String,

        /// A code associated with the error. Useful for programmatically identifying
        /// the nature of the error, and may be helpful in debugging or categorizing different types of execution errors.
        code: String
    },

    NoSmartContractResult
}

impl From<SimulationError> for ExecutorError {
    fn from(value: SimulationError) -> Self {
        ExecutorError::Simulation(value)
    }
}