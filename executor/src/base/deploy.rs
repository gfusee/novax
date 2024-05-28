use std::sync::Arc;

use async_trait::async_trait;
use multiversx_sc::types::CodeMetadata;
use num_bigint::BigUint;
use tokio::sync::Mutex;

use novax_data::{Address, NativeConvertible};

use crate::call_result::CallResult;
use crate::error::executor::ExecutorError;
use crate::TopDecodeMulti;

/// A trait defining the contract for executing smart contract deployment operations asynchronously.
#[async_trait]
pub trait DeployExecutor: Send + Sync {
    /// Executes a smart contract deployment step asynchronously.
    ///
    /// # Type Parameters
    ///
    /// * `OriginalResult` - The result type expected from the smart contract deployment.
    ///   Must implement `TopEncodeMulti`, `Send`, and `Sync`.
    /// * `S` - The type encapsulating the smart contract deployment step.
    ///   Must implement `AsMut<TypedScDeploy<OriginalResult>>` and `Send`.
    ///
    /// # Parameters
    ///
    /// * `sc_deploy_step` - The smart contract deployment step to be executed.
    ///
    /// # Returns
    ///
    /// A `Result` with an empty `Ok(())` value for success, or `Err(ExecutorError)` for failure.
    async fn sc_deploy<
        OutputManaged
    >(
        &mut self,
        bytes: Vec<u8>,
        code_metadata: CodeMetadata,
        egld_value: BigUint,
        arguments: Vec<Vec<u8>>,
        gas_limit: u64
    ) -> Result<(Address, CallResult<OutputManaged::Native>), ExecutorError>
        where
            OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync;
}

/// An implementation of `DeployExecutor` for `Arc<Mutex<T>>` where `T: DeployExecutor`.
/// This wrapper allows for thread-safe, shared ownership of a deploy executor.
#[async_trait]
impl<T: DeployExecutor> DeployExecutor for Arc<Mutex<T>> {
    /// Executes a smart contract deployment step asynchronously, delegating to the inner `DeployExecutor`.
    async fn sc_deploy<
        OutputManaged
    >(
        &mut self,
        bytes: Vec<u8>,
        code_metadata: CodeMetadata,
        egld_value: BigUint,
        arguments: Vec<Vec<u8>>,
        gas_limit: u64
    ) -> Result<(Address, CallResult<OutputManaged::Native>), ExecutorError>
        where
            OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
    {
        {
            let mut locked = self.lock().await;
            locked.sc_deploy::<OutputManaged>(bytes, code_metadata, egld_value, arguments, gas_limit).await
        }
    }
}
