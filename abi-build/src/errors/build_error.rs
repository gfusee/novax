use crate::abi::error::AbiError;
use crate::generator::generator_error::GeneratorError;

#[derive(Clone, Debug)]
pub enum BuildError {
    Abi(AbiError),
    Generator(GeneratorError)
}