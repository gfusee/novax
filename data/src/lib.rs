//! `novax-data` is a crate designed to facilitate data handling and conversions in blockchain-based applications.
//!
//! This crate provides a robust set of types and utility functions to ease the conversion and management 
//! of data structures when working with MultiversX virtual machine and the associated blockchain technology.
//! It encompasses common patterns for data conversion and parsing, especially around address representations,
//! and error handling.
//!
//! # Core Concepts
//!
//! - **Native and Managed Type Conversion**:
//!   `NativeConvertible` and `ManagedConvertible` are traits provided to bridge the gap between complex smart contract types
//!   managed by the MultiversX virtual machine and common Rust types. They facilitate seamless conversions back and forth,
//!   supporting a variety of scenarios such as converting a `String` to a `ManagedBuffer` or `TokenIdentifier`.
//!
//! - **Address Handling**:
//!   The `Address` struct along with its associated methods simplify the operations and transformations
//!   required when dealing with address representations on the blockchain.
//!
//! - **Data Parsing and Error Handling**:
//!   Utility functions like `parse_query_return_string_data` and `parse_query_return_bytes_data` are provided to
//!   parse and decode data from blockchain queries. Comprehensive error types like `DataError`, `AddressError`, and
//!   `UtilsError` centralize error handling, making error propagation and management straightforward.
//!
//! # Usage
//!
//! Most of the time, developers won't have to include `novax-data` directly as a dependency. Its primary purpose is
//! to serve as a foundational utility crate used by other "novax" crates such as "novax", "novax-executor", "novax-token", etc.
//! However, if direct usage is required:
//!
//! ```rust
//! use novax_data::{Address, NativeConvertible, ManagedConvertible, parse_query_return_string_data, DataError};
//!
//! // ... your code here ...
//! ```
//!
//! For detailed examples and usage of each type and utility function, refer to their respective module and function documentation.
//!
//! # Modules
//! - `types`: Defines core types like `Address`, and the conversion traits `NativeConvertible` and `ManagedConvertible`.
//! - `constants`: (Further details can be provided as needed)
//! - `error`: Centralizes error definitions including `DataError`, `AddressError`, and `UtilsError` for robust error handling.
//! - `utils`: Provides utility functions for data parsing and other common operations.
//!
//! For a deep dive into each module and to understand the various types, traits, and functions provided,
//! navigate through the module documentation below.
//!
// TODO #![warn(missing_docs)]

mod types;
mod constants;
mod error;
mod utils;

pub use error::*;
pub use crate::types::native::NativeConvertible;
pub use crate::types::managed::ManagedConvertible;
pub use crate::types::address::Address;
pub use crate::types::payment::Payment;
pub use crate::utils::parse_query_return_data::parse_query_return_string_data;
pub use crate::utils::parse_query_return_data::parse_query_return_bytes_data;