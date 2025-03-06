//! `novax-executor` is a crate designed to facilitate the interaction between your Rust application and a blockchain network.
//! Its main purpose is to be used by other "novax" crates like "novax", however, it's designed in a way that developers can also use it for mocking purposes during testing.
//! It abstracts the complexities involved in querying the blockchain, executing transactions, and deploying smart contracts,
//! thereby providing a streamlined interface for developers.
//!
//! The crate offers several core abstractions and implementations to interact with a blockchain:
//! - **Executor Abstractions**:
//!   - `QueryExecutor`: An asynchronous trait for executing queries on the blockchain.
//!   - `TransactionExecutor`: An asynchronous trait for executing transactions on the blockchain.
//!   - `DeployExecutor`: An asynchronous trait for deploying smart contracts on the blockchain.
//!
//! - **Network Interaction**:
//!   - `BlockchainInteractor`: A trait abstracting over a blockchain interactor to allow mocking.
//!   - `BlockchainProxy`: A trait abstracting the communication proxy to allow mocking, providing methods to interact with the blockchain gateway.
//!   - `ProxyQueryExecutor` and `QueryNetworkExecutor`: Implementations for executing queries on the blockchain.
//!   - `NetworkExecutor` and `BaseTransactionNetworkExecutor`: Implementations for executing transactions on the blockchain.
//!
//! - **Mocking Framework**:
//!   - `MockExecutor`: A structure to help mock blockchain interactions during testing using the MultiversX Rust Testing Framework.
//!   - `StandardMockExecutor`: An extension of `MockExecutor` providing standard mocking behaviors.
//!
//! - **Dummy Framework**:
//!   - `DummyExecutor`, `DummyTransactionExecutor`, and `DummyDeployExecutor`: Implementations to assist in testing and development without actual blockchain interaction.
//!
//! - **Utility Types and Traits**:
//!   - `SendableTransaction` and `SendableTransactionConvertible`: Utility types and traits to facilitate transaction handling.
//!
//! - **Error Handling**:
//!   - `ExecutorError`: A comprehensive enumeration of errors that could occur during blockchain interaction, encompassing data errors, network query errors, and mock deploy errors.
//!
//! The abstraction layers provided by this crate are designed to make it easy to implement mock or dummy executors, allowing for thorough testing and development
//! without requiring a live blockchain network. This is especially helpful in early stages of development or in testing scenarios where the blockchain's state
//! or behavior needs to be controlled precisely.
//!
//! The architecture also allows for the easy extension of the executor framework to support additional blockchain networks or custom interaction patterns.
//!
//! # Features
//! - `async-trait`: This crate uses the [`async-trait`](https://crates.io/crates/async-trait) crate to allow for async trait methods,
//!   enabling asynchronous blockchain interaction.
//!
//! # Error Handling
//! Error handling in `novax-executor` is comprehensive and designed to cover a range of issues that might arise while interacting with the blockchain.
//! See [`ExecutorError`](enum.ExecutorError.html), [`NetworkQueryError`](enum.NetworkQueryError.html), and [`MockDeployError`](enum.MockDeployError.html) for more details.

// TODO #![warn(missing_docs)]

mod error;
mod base;
mod network;
mod mocking;
mod dummy;
mod utils;

pub use error::executor::ExecutorError;
pub use error::network::NetworkQueryError;
pub use error::mock_deploy::MockDeployError;
pub use error::mock_transaction::MockTransactionError;
pub use error::gateway::GatewayError;
pub use error::transaction::TransactionError;
pub use error::simulation::SimulationError;

pub use base::query::QueryExecutor;
pub use base::transaction::TransactionExecutor;
pub use base::deploy::DeployExecutor;

pub use network::query::executor::ProxyQueryExecutor;
pub use network::query::executor::QueryNetworkExecutor;
pub use network::utils::wallet::Wallet;
pub use network::transaction::executor::NetworkExecutor;
pub use network::transaction::executor::BaseTransactionNetworkExecutor;
pub use network::transaction::interactor::BlockchainInteractor;
pub use network::transaction::interactor::TransactionRefreshStrategy;
pub use network::query::proxy::BlockchainProxy;
pub use network::simulate::SimulationNetworkExecutor;
pub use network::simulate::BaseSimulationNetworkExecutor;
pub use network::models::simulate::request::SimulationGatewayRequest;
pub use network::models::simulate::response::SimulationGatewayResponse;
pub use network::query::models::request::VmValuesQueryRequest;
pub use network::query::models::response::VmValuesQueryResponseData;
pub use network::query::models::response::VmValuesQueryResponseDataData;
pub use network::transaction::models::transaction_on_network::TransactionOnNetworkResponse;
pub use network::transaction::models::transaction_on_network::TransactionOnNetwork;
pub use network::transaction::models::transaction_on_network::TransactionOnNetworkTransaction;
pub use network::transaction::models::transaction_on_network::TransactionOnNetworkTransactionSmartContractResult;
pub use network::transaction::models::transaction_on_network::TransactionOnNetworkTransactionLogs;
pub use network::transaction::models::transaction_on_network::TransactionOnNetworkTransactionLogsEvents;

pub use mocking::executor::StandardMockExecutor;
pub use mocking::executor::MockExecutor;

pub use dummy::transaction::DummyExecutor;
pub use dummy::transaction::DummyTransactionExecutor;
pub use dummy::transaction::DummyDeployExecutor;

pub use utils::transaction::data::SendableTransaction;
pub use utils::transaction::data::SendableTransactionConvertible;
pub use utils::transaction::token_transfer::TokenTransfer;
pub use utils::transaction::call_result;
pub use utils::transaction::results;
pub use utils::transaction::normalization::NormalizationInOut;

pub use multiversx_sc_scenario::ScenarioWorld;
pub use multiversx_sc_scenario::scenario_model::{ScCallStep, ScDeployStep, ScQueryStep, SetStateStep, Account, TxQuery, TxResponse, TypedScDeploy, TypedScQuery, TypedResponse};
pub use multiversx_sc::codec::TopDecodeMulti;