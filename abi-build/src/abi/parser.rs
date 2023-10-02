use std::fs;
use std::path::Path;
use crate::abi::result::Abi;
use crate::abi::error::AbiError::{FileNotFound, JsonParseFailed};
use crate::errors::build_error::BuildError;

pub fn parse_abi_file<P>(file_path: &P) -> Result<Abi, BuildError>
where
    P: AsRef<Path>
{
    let Ok(file_content) = fs::read_to_string(file_path) else { return Err(FileNotFound.into()) };
    let Ok(abi) = serde_json::from_str(&file_content) else { return Err(JsonParseFailed.into()) };

    Ok(abi)
}