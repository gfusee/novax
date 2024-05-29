use std::fs::read;
use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::code::bytes::AsBytesValue;
use crate::errors::{CodeError, NovaXError};

/// Represents a file containing code, identified by its path.
pub struct FileCode(PathBuf);

#[async_trait]
impl AsBytesValue for &FileCode {
    /// Asynchronously reads the content of the file identified by the path contained in `self`,
    /// converting the file content into a `BytesValue`.
    ///
    /// # Errors
    ///
    /// Returns a `CodeError` wrapped in a `NovaXError` if the file cannot be read.
    async fn into_bytes_value(self) -> Result<Vec<u8>, NovaXError> {
        read(&self.0)
            .map_err(|_| CodeError::UnableToReadCodeFromFile.into())
    }
}

#[async_trait]
impl AsBytesValue for &str {
    /// Asynchronously reads the content of the file identified by `self` as a path,
    /// converting the file content into a `BytesValue`.
    ///
    /// This implementation constructs a `FileCode` from `self` and delegates the task to the
    /// `FileCode` implementation of `AsBytesValue`.
    ///
    /// # Errors
    ///
    /// Returns a `CodeError` wrapped in a `NovaXError` if the file cannot be read.
    async fn into_bytes_value(self) -> Result<Vec<u8>, NovaXError> {
        FileCode(Path::new(self).to_path_buf()).into_bytes_value().await
    }
}