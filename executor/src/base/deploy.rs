use std::sync::Arc;
use async_trait::async_trait;
use multiversx_sc::codec::TopEncodeMulti;
use multiversx_sc::types::CodeMetadata;
use multiversx_sc_scenario::scenario_model::TypedScDeploy;
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

    /// Indicates whether to skip deserialization during the deployment execution.
    ///
    /// This could be useful in cases where deserialization is either unnecessary or could cause errors,
    /// for example, with the `DummyExecutor`.
    ///
    /// # Returns
    ///
    /// A `bool` indicating whether deserialization should be skipped.
    async fn should_skip_deserialization(&self) -> bool; // TODO: remove
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

    /// Indicates whether to skip deserialization during the deployment execution, delegating to the inner `DeployExecutor`.
    async fn should_skip_deserialization(&self) -> bool {
        {
            // Locking here can lead to some performance penalty. A potential solution may be to use
            // another type other than Mutex, like RwLock.
            let locked = self.lock().await;
            locked.should_skip_deserialization().await
        }
    }
}
