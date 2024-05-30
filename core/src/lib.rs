//! # NovaX Crate
//!
//! `novax` is a Rust crate dedicated to generating clients at compile time to interact with smart contracts on a blockchain.
//! The crate automatically generates necessary structures and methods from ABIs (Application Binary Interface) to provide a seamless and type-safe way to interact with smart contracts.
//! The crate's functionality is driven by the environment variable `NOVAX_PATH` which should point to a directory containing a sub-directory named `abis` where the ABIs are located.
//!
//! ## Setup
//!
//! To use `novax`, set up the environment variable `NOVAX_PATH` in your `.cargo/config.toml` file as follows:
//!
//! ```toml
//! [env]
//! NOVAX_PATH={ value = ".novax", relative = true }
//! ```
//!
//! Here, `.novax` is a folder at the project's root containing a sub-directory named `abis` where the ABIs are placed.
//!
//! ## Modules
//!
//! - `caching`: Provides caching strategies to optimize smart contract queries.
//! - `errors`: Centralized module for handling various error types encountered within the `novax` crate's operations.
//! - `transaction`: Contains essential structs and types for handling blockchain transactions.
//! - `code`: Facilitates reading and handling of smart contract code.
//! - `account`: Provides structures and functionalities for handling and obtaining account information.
//!
//! The crate also includes a generated module from a file in the `OUT_DIR` directory, which contains structures and methods derived from the ABIs.
//!
//! ## Key Features
//!
//! - `CachingStrategy` and `CachingNone`: Facilitate caching behavior during query operations, defined in the `caching` module.
//! - `CallResult` and `TokenTransfer`: Handle transaction responses and token transfers respectively, located in the `transaction` module.
//! - `AsBytesValue`, `FileCode`, and `DeployData`: Essential for handling smart contract code and deployment data, defined in the `code` module.
//! - `AccountInfos` and `from_address` function: Assist in obtaining and handling account-related information, found in the `account` module.
//! - Error Handling: Various error types such as `CodeError`, `CachingError`, `AccountError`, and `DateError` are defined in the `errors` module to cover different error scenarios.
//!
//! ## Usage
//!
//! Once the `NOVAX_PATH` environment variable is set up and the ABIs are placed in the specified directory,
//! the `novax` crate automatically generates the required client code when the project is built.
//! This generated code can then be used to interact with the smart contracts as per the methods and structures defined in the ABIs.
//!
//! ## Compile-Time Code Generation
//!
//! The `novax` crate utilizes Rust's procedural macro facilities to perform code generation at compile time.
//! This ensures that the generated code is type-safe, efficient, and ready-to-use right out of the box.
#![warn(missing_docs)]

/// The `caching` module provides caching strategies to optimize smart contract queries.
pub mod caching;

/// The `errors` module centralizes various error types encountered within the `novax` crate's operations.
pub mod errors;

/// The `code` module facilitates reading and handling of smart contract code.
pub mod code;

/// The `account` module provides structures and functionalities for handling and obtaining account information.
pub mod account;

/// The `utils` module provides some helpers to make the framework working.
pub mod utils;



// Include the generated client code from the output directory.
include!(concat!(env!("OUT_DIR"), "/generated_lib.rs"));

pub use multiversx_sdk::data::address::Address as SDKAddress;
pub use multiversx_sdk::data::vm::VMOutputApi;
pub use novax_executor::Wallet;
pub use novax_data::Address;
pub use multiversx_sc::types::CodeMetadata;
use multiversx_sc_scenario::imports::StaticApi;

pub type EgldOrMultiEsdtPayment = multiversx_sc::types::EgldOrMultiEsdtPayment<StaticApi>;

/// The `executor` module provides re-exports of functionalities from the `novax_executor` crate.
///
/// This module serves as a bridge to the `novax_executor` crate, allowing access to its provided
/// functionalities for executing transactions and queries against the blockchain.
pub mod executor {
    pub use novax_executor::*;
}

/// The `data` module provides re-exports of functionalities from the `novax_data` crate.
///
/// This module acts as an interface to the `novax_data` crate, facilitating access to its
/// provided features for handling data structures, storage mechanisms, and other data-related
/// operations used within the blockchain context.
pub mod data {
    pub use novax_data::*;
}