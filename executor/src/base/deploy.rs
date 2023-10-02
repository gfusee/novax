use std::sync::Arc;
use async_trait::async_trait;
use multiversx_sc::codec::TopEncodeMulti;
use multiversx_sc_scenario::scenario_model::TypedScDeploy;
use tokio::sync::Mutex;
use crate::error::executor::ExecutorError;

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
    async fn sc_deploy<OriginalResult, S>(&mut self, sc_deploy_step: S) -> Result<(), ExecutorError>
        where
            OriginalResult: TopEncodeMulti + Send + Sync,
            S: AsMut<TypedScDeploy<OriginalResult>> + Send;

    /// Indicates whether to skip deserialization during the deployment execution.
    ///
    /// This could be useful in cases where deserialization is either unnecessary or could cause errors,
    /// for example, with the `DummyExecutor`.
    ///
    /// # Returns
    ///
    /// A `bool` indicating whether deserialization should be skipped.
    async fn should_skip_deserialization(&self) -> bool;
}

/// An implementation of `DeployExecutor` for `Arc<Mutex<T>>` where `T: DeployExecutor`.
/// This wrapper allows for thread-safe, shared ownership of a deploy executor.
#[async_trait]
impl<T: DeployExecutor> DeployExecutor for Arc<Mutex<T>> {
    /// Executes a smart contract deployment step asynchronously, delegating to the inner `DeployExecutor`.
    async fn sc_deploy<OriginalResult, S>(&mut self, sc_deploy_step: S) -> Result<(), ExecutorError>
        where
            OriginalResult: TopEncodeMulti + Send + Sync,
            S: AsMut<TypedScDeploy<OriginalResult>> + Send
    {
        {
            let mut locked = self.lock().await;
            locked.sc_deploy(sc_deploy_step).await
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
