use crate::error::executor::ExecutorError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum NetworkQueryEventsError {
    ErrorWhileSendingQuery { reason: String },
    ResponseDoesntHaveHitsField { response: String },
    HitDoesntHaveSourceField { hit: String },
    CannotDeserializeHitSource { hit: String, reason: String },
    CannotDecodeHexEventIdentifier { event_identifier: String, reason: String },
    CannotGetUtf8EventIdentifierFromBytes { event_identifier_bytes: Vec<u8>, reason: String },
    CannotDecodeHexTopic { topic: String, reason: String },
    CannotDeserializeTopicToContractType { topics: Vec<String> },
    Other { id: String, reason: String }, // For use to crates using this one as dependency
}

impl From<NetworkQueryEventsError> for ExecutorError {
    fn from(value: NetworkQueryEventsError) -> Self {
        ExecutorError::NetworkQueryEvents(value)
    }
}