use std::sync::Arc;

use async_trait::async_trait;
use multiversx_sc::codec::TopDecodeMulti;
use num_bigint::BigUint;
use tokio::sync::Mutex;

use novax_data::{Address, NativeConvertible};

use crate::call_result::CallResult;
use crate::error::executor::ExecutorError;
use crate::utils::transaction::token_transfer::TokenTransfer;

/// A trait defining the necessary operations for executing smart contract transactions.
///
/// Implementations of this trait can vary based on the specific environment (e.g., real blockchain, mock blockchain).
#[async_trait]
pub trait TransactionExecutor: Send + Sync {
    /// Executes a smart contract call with the specified parameters.
    ///
    /// # Parameters
    /// - `sc_call_step`: A mutable reference to the typed smart contract call step.
    ///
    /// # Type Parameters
    /// - `OriginalResult`: The type of the result expected from the smart contract call. Must implement the `Send` trait.
    ///
    /// # Returns
    /// - A `Result` with an empty `Ok(())` value if the call is successful, or an `Err(ExecutorError)` if the call fails.
    async fn sc_call<OutputManaged>(
        &mut self,
        to: &Address,
        function: String,
        arguments: &[Vec<u8>],
        gas_limit: u64,
        egld_value: BigUint,
        esdt_transfers: Vec<TokenTransfer>
    ) -> Result<CallResult<OutputManaged::Native>, ExecutorError>
        where
            OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync;
}

/// An implementation of `TransactionExecutor` trait for types wrapped in `Arc<Mutex<T>>`.
///
/// This implementation allows shared access to a transaction executor instance across multiple threads.
#[async_trait]
impl<T: TransactionExecutor> TransactionExecutor for Arc<Mutex<T>> {
    /// Executes a smart contract call using the underlying `TransactionExecutor` implementation.
    async fn sc_call<OutputManaged>(
        &mut self,
        to: &Address,
        function: String,
        arguments: &[Vec<u8>],
        gas_limit: u64,
        egld_value: BigUint,
        esdt_transfers: Vec<TokenTransfer>
    ) -> Result<CallResult<OutputManaged::Native>, ExecutorError>
        where
            OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
    {
        {
            // Acquire a lock to ensure exclusive access to the executor during the call execution.
            let mut executor = self.lock().await;
            executor.sc_call::<OutputManaged>(
                to,
                function,
                arguments,
                gas_limit,
                egld_value,
                esdt_transfers
            ).await
        }
    }
}
