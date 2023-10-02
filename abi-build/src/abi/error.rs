use crate::errors::build_error::BuildError;

#[derive(Clone, Debug)]
pub enum AbiError {
    FileNotFound,
    FileContentParseFailed,
    JsonParseFailed,
    UnknownType
}

impl From<AbiError> for BuildError {
    fn from(value: AbiError) -> Self {
        BuildError::Abi(value)
    }
}