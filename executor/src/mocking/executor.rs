use std::mem;
use std::ops::Deref;
use std::sync::Arc;
use async_trait::async_trait;
use multiversx_sc::api::{HandleTypeInfo, VMApi};
use multiversx_sc::codec::{TopDecodeMulti, TopEncodeMulti};
use multiversx_sc::imports::{TxEnv, TxFrom, TxGas, TxPayment, TxTo, TxTypedCall};
use num_bigint::BigUint;
use crate::{ScCallStep, ScDeployStep, ScQueryStep, TxQuery, TypedScCall, TypedScDeploy, TypedScQuery};
use crate::ScenarioWorld;
use tokio::sync::{Mutex, MutexGuard};
use novax_data::Address;
use novax_data::NativeConvertible;
use novax_data::parse_query_return_bytes_data;
use crate::base::deploy::DeployExecutor;
use crate::base::query::QueryExecutor;
use crate::base::transaction::TransactionExecutor;
use crate::call_result::CallResult;
use crate::error::executor::ExecutorError;
use crate::error::mock_deploy::MockDeployError;
use crate::utils::transaction::token_transfer::TokenTransfer;

/// A convenient type alias for `MockExecutor` with `String` as the generic type.
pub type StandardMockExecutor = MockExecutor<String>;

/// A structure to execute smart contract queries, transactions, and deployments in a mocked blockchain environment.
///
/// This executor utilizes the scenario engine from the MultiversX Rust Testing Framework for executing smart contract interactions.
#[derive(Clone)]
pub struct MockExecutor<A>
    where
        A: Deref + Send + Sync,
        Address: for<'a> From<&'a A::Target>
{
    /// The mocked world where the smart contract interactions are executed.
    world: Arc<Mutex<ScenarioWorld>>,
    /// Optional caller address. If not provided, the executor uses the address from the smart contract call or deployment request.
    opt_caller: Option<A>,
}

impl<A> MockExecutor<A>
    where
        A: Deref + Send + Sync,
        Address: for<'a> From<&'a A::Target>
{
    /// Constructs a new `MockExecutor` with the specified mocked world and an optional caller address.
    ///
    /// # Parameters
    /// - `world`: The mocked world where the smart contract interactions are executed.
    /// - `opt_caller`: Optional caller address.
    ///
    /// # Returns
    /// A new instance of `MockExecutor`.
    pub fn new(world: Arc<Mutex<ScenarioWorld>>, opt_caller: Option<A>) -> MockExecutor<A> {
        MockExecutor {
            world,
            opt_caller,
        }
    }
}

#[async_trait]
impl<A> TransactionExecutor for MockExecutor<A>
    where
        A: Deref + Send + Sync,
        Address: for<'a> From<&'a A::Target>
{
    /// Executes a smart contract call within a mocked environment.
    ///
    /// This method extracts or determines the caller's address, performs a smart contract call,
    /// and updates the world state accordingly, all within a controlled, mocked environment.
    ///
    /// # Parameters
    /// - `sc_call_step`: A mutable reference to a `TypedScCall` object representing the smart contract call step.
    ///
    /// # Type Parameters
    /// - `OriginalResult`: The type of the result expected from the smart contract call. Must implement the `Send` trait.
    ///
    /// # Returns
    /// - A `Result` object with an empty `Ok(())` value if the call is successful,
    ///   or an `Err(ExecutorError)` if the call fails for any reason.
    async fn sc_call<OutputManaged>(
        &mut self,
        to: &Address,
        function: &str,
        arguments: &[&[u8]],
        gas_limit: u64,
        egld_value: &BigUint,
        esdt_transfers: &[TokenTransfer]
    ) -> Result<CallResult<OutputManaged::Native>, ExecutorError>
        where
            OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
    {
        /*
        let caller: Address = if let Some(caller) = self.opt_caller.as_deref() {
            caller.into()
        } else {
            (&sc_call_step.sc_call_step.tx.to.value).into()
        };

        let owned_sc_call_step = mem::replace(sc_call_step, ScCallStep::new().into());
        *sc_call_step = owned_sc_call_step.from(&caller);

        {
            let mut world = self.world.lock().await;
            world.sc_call(sc_call_step);
        }

        Ok(())

         */

        todo!()
    }

    /// Specifies whether deserialization should be skipped during the smart contract call execution.
    ///
    /// In the context of the mocked environment, deserialization is not skipped,
    /// hence this method returns `false`.
    ///
    /// # Returns
    /// - A boolean value `false`, indicating that deserialization should not be skipped.
    async fn should_skip_deserialization(&self) -> bool {
        false
    }
}

/// Mock implementation of the `DeployExecutor` trait for testing and development purposes.
/// This implementation uses a mock executor to simulate the deployment of smart contracts
/// on the blockchain without actually interacting with a real blockchain network.
///
/// The `MockExecutor` struct encapsulates the state and behavior necessary for simulating
/// blockchain interactions.
///
/// # Type Parameters
///
/// * `A`: A type implementing `Deref`, `Send`, and `Sync`. This type is used to derive an
///   `Address` type instance representing a blockchain address.
#[async_trait]
impl<A> DeployExecutor for MockExecutor<A>
    where
        A: Deref + Send + Sync,
        Address: for<'a> From<&'a A::Target>
{
    /// Asynchronously deploys a smart contract to the mock blockchain environment.
    ///
    /// # Type Parameters
    ///
    /// * `OriginalResult`: Represents the result type expected from the smart contract deployment.
    ///   This type must implement `TopEncodeMulti`, `Send`, and `Sync`.
    /// * `S`: Represents the type encapsulating the smart contract deployment step.
    ///   This type must implement `AsMut<TypedScDeploy<OriginalResult>>` and `Send`.
    ///
    /// # Parameters
    ///
    /// * `sc_deploy_step`: A mutable reference to the smart contract deployment step to be executed.
    ///
    /// # Returns
    ///
    /// A `Result` with an empty `Ok(())` value indicating success, or an `Err(ExecutorError)` indicating failure,
    /// specifically a `MockDeployError::WalletAddressNotPresent` error if the wallet address is not present.
    async fn sc_deploy<OriginalResult>(&mut self, sc_deploy_step: &mut TypedScDeploy<OriginalResult>) -> Result<(), ExecutorError>
        where
            OriginalResult: TopEncodeMulti + Send + Sync,
    {
        let caller: Address = {
            let Some(caller) = self.opt_caller.as_deref() else {
                return Err(ExecutorError::MockDeploy(MockDeployError::WalletAddressNotPresent))
            };

            caller.into()
        };

        let sc_deploy_step = sc_deploy_step.as_mut();

        let owned_sc_deploy_step = mem::replace(sc_deploy_step, ScDeployStep::new());
        *sc_deploy_step = owned_sc_deploy_step.from(&caller);

        {
            let mut world = self.world.lock().await;
            world.sc_deploy(sc_deploy_step);
        }

        Ok(())
    }

    /// Specifies whether deserialization should be skipped during the deployment execution.
    /// In this implementation, deserialization is not skipped.
    ///
    /// # Returns
    ///
    /// A `bool` value of `false`, indicating that deserialization should not be skipped.
    async fn should_skip_deserialization(&self) -> bool {
        false
    }
}

/// The `MockExecutor` implementation for the `QueryExecutor` trait, used to simulate smart contract queries in a mock environment.
/// This struct is typically utilized in testing and development scenarios where interaction with a real blockchain is undesirable or unnecessary.
///
/// # Type Parameters
///
/// * `A`: A type that implements `Clone`, `Deref`, `Send`, and `Sync` traits. This type is used to derive an `Address` instance representing a blockchain address.
#[async_trait]
impl<A> QueryExecutor for MockExecutor<A>
    where
        A: Clone + Deref + Send + Sync,
        Address: for<'a> From<&'a A::Target>
{
    /// Executes a simulated smart contract query in the mock environment.
    ///
    /// # Type Parameters
    ///
    /// * `OutputManaged`: The type representing the expected output of the smart contract query. It should implement `TopDecodeMulti`, `NativeConvertible`, `Send`, and `Sync`.
    ///
    /// # Parameters
    ///
    /// * `request`: A reference to the `ScCallStep` detailing the smart contract query to be executed.
    ///
    /// # Returns
    ///
    /// * `Result<OutputManaged::Native, ExecutorError>`: On successful execution, returns a `Result` containing the native converted query output.
    ///   On failure, returns a `Result` containing an `ExecutorError`.
    async fn execute<OutputManaged>(
        &self,
        to: &Address,
        function: &str,
        arguments: &[&[u8]],
        egld_value: &BigUint,
        esdt_transfers: &[TokenTransfer]
    ) -> Result<OutputManaged::Native, ExecutorError>
        where
            OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
    {
        /*
        // Convert the smart contract query step to a call step.
        let query = convert_sc_query_step_to_call_step(request);
        // Create a TypedScQuery from the query.
        let mut typed = TypedScQuery::<OutputManaged>::from(query);
        {
            // Lock the mock world state for exclusive access.
            let mut world: MutexGuard<ScenarioWorld> = self.world.lock().await;
            // Execute the smart contract query in the mock world.
            world.sc_query(&mut typed);
        }
        // Retrieve the response from the typed query.
        let response = typed.response();
        // Clone the output from the response.
        let mut out = response.out.clone();

        // Parse the query return bytes data.
        let parsed = parse_query_return_bytes_data::<OutputManaged>(&mut out)?;
        // Convert the parsed data to its native type and return it.
        Ok(parsed.to_native())

         */

        todo!()
    }
}

/// Converts a smart contract query step to a call step.
///
/// This conversion is needed to accommodate the types expected by the scenario engine from the MultiversX Rust Testing Framework.
fn convert_sc_query_step_to_call_step(query_step: &ScCallStep) -> ScQueryStep {
    let query_tx = TxQuery {
        to: query_step.tx.to.clone(),
        function: query_step.tx.function.clone(),
        arguments: query_step.tx.arguments.clone(),
    };

    ScQueryStep {
        id: query_step.id.clone(),
        tx_id: query_step.tx_id.clone(),
        explicit_tx_hash: query_step.explicit_tx_hash.clone(),
        comment: query_step.comment.clone(),
        tx: Box::new(query_tx),
        expect: query_step.expect.clone(),
        response: query_step.response.clone(),
    }
}