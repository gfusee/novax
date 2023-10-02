use crate::errors::build_error::BuildError;

#[derive(Clone, Debug)]
pub enum GeneratorError {
    UnableToFormatRustCode,
    TypeNotFoundForInput,
}

impl From<GeneratorError> for BuildError {
    fn from(value: GeneratorError) -> Self {
        BuildError::Generator(value)
    }
}