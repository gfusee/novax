use async_trait::async_trait;

use crate::errors::NovaXError;

/// An asynchronous trait defining a type that can be converted into a `BytesValue`.
/// This trait is particularly used when deploying contracts, often by reading a .wasm (WebAssembly) file.
#[async_trait]
pub trait AsBytesValue {
    /// Asynchronously converts `self` into a `BytesValue`.
    ///
    /// This method is intended to facilitate the conversion of different types into a `BytesValue`,
    /// which can be particularly useful when dealing with contract deployment or other scenarios
    /// where a byte representation of a type is required.
    ///
    /// # Returns
    ///
    /// A `Result` wrapping a `BytesValue` if the conversion is successful, or an `Err` wrapping
    /// a `NovaXError` if the conversion fails.
    async fn into_bytes_value(self) -> Result<Vec<u8>, NovaXError>;
}
