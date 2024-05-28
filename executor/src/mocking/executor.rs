use std::mem;
use std::ops::Deref;
use std::sync::Arc;

use async_trait::async_trait;
use multiversx_sc::codec::{TopDecodeMulti, TopEncodeMulti};
use multiversx_sc::imports::{CodeMetadata, ReturnsNewAddress};
use multiversx_sc::types::{MultiValueEncoded, ReturnsRawResult};
use multiversx_sc_scenario::imports::{Bech32Address, BytesValue};
use multiversx_sc_scenario::ScenarioTxRun;
use num_bigint::BigUint;
use tokio::sync::Mutex;

use novax_data::Address;
use novax_data::NativeConvertible;

use crate::{ScCallStep, ScDeployStep, ScQueryStep, TxQuery, TypedScDeploy};
use crate::base::deploy::DeployExecutor;
use crate::base::query::QueryExecutor;
use crate::base::transaction::TransactionExecutor;
use crate::call_result::CallResult;
use crate::error::executor::ExecutorError;
use crate::error::mock_deploy::MockDeployError;
use crate::error::mock_transaction::MockTransactionError;
use crate::error::transaction::TransactionError;
use crate::ScenarioWorld;
use crate::utils::transaction::deploy::get_deploy_call_input;
use crate::utils::transaction::token_transfer::TokenTransfer;
use crate::utils::transaction::transfers::get_egld_or_esdt_transfers;

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
        function: String,
        arguments: Vec<Vec<u8>>,
        gas_limit: u64,
        egld_value: BigUint,
        esdt_transfers: Vec<TokenTransfer>
    ) -> Result<CallResult<OutputManaged::Native>, ExecutorError>
        where
            OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
    {
        let mut world = self.world.lock().await;

        let transfers = get_egld_or_esdt_transfers(
            egld_value,
            esdt_transfers
        )?;

        let Some(caller) = self.opt_caller.as_ref() else {
            return Err(MockTransactionError::CallerAddressNotPresent.into())
        };

        let mut tx = world.tx()
            .from(Bech32Address::from_bech32_string(Address::from(caller).to_bech32_string()?))
            .to(Bech32Address::from_bech32_string(to.to_bech32_string()?))
            .raw_call(function)
            .with_gas_limit(gas_limit)
            .egld_or_multi_esdt(transfers)
            .returns(ReturnsRawResult);

        for argument in arguments {
            tx = tx.argument(&argument);
        }

        let raw_result_managed = tx.run();
        let mut raw_result: Vec<Vec<u8>> = raw_result_managed
            .into_vec()
            .iter()
            .map(|buffer| buffer.to_boxed_bytes().into_vec())
            .collect();

        let output_managed = OutputManaged::multi_decode(&mut raw_result).unwrap(); // TODO: no unwrap

        let call_result = CallResult {
            response: Default::default(),
            result: Some(output_managed.to_native()),
        };

        Ok(call_result)
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
        let mut world = self.world.lock().await;

        let Some(caller) = self.opt_caller.as_ref() else {
            return Err(MockDeployError::CallerAddressNotPresent.into())
        };

        let mut tx = world.tx()
            .from(Bech32Address::from_bech32_string(Address::from(caller).to_bech32_string()?))
            .raw_deploy()
            .code_metadata(code_metadata)
            .code(BytesValue::from(bytes))
            .with_gas_limit(gas_limit)
            .egld(multiversx_sc::types::BigUint::from(egld_value))
            .returns(ReturnsNewAddress)
            .returns(ReturnsRawResult);

        for argument in arguments {
            tx = tx.argument(&argument);
        }

        let (new_address, raw_result_managed) = tx.run();
        let mut raw_result: Vec<Vec<u8>> = raw_result_managed
            .into_vec()
            .iter()
            .map(|buffer| buffer.to_boxed_bytes().into_vec())
            .collect();

        let output_managed = OutputManaged::multi_decode(&mut raw_result).unwrap(); // TODO: no unwrap

        let call_result = CallResult {
            response: Default::default(),
            result: Some(output_managed.to_native()),
        };

        Ok((Address::from_bytes(*new_address.as_array()), call_result))
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
        function: String,
        arguments: Vec<Vec<u8>>,
        egld_value: BigUint,
        esdt_transfers: Vec<TokenTransfer>
    ) -> Result<OutputManaged::Native, ExecutorError>
        where
            OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
    {
        let mut world = self.world.lock().await;

        // TODO: use from when run() becomes available with it in the MvX Rust SDK
        // TODO: use payments when run() becomes available with them in the MvX Rust SDK
        // let from = self.opt_caller.as_ref().map(|a| Address::from(a)).unwrap_or_else(|| to.clone());

        let mut tx = world.query()
            .to(Bech32Address::from_bech32_string(to.to_bech32_string()?))
            .raw_call(function)
            .returns(ReturnsRawResult);

        for argument in arguments {
            tx = tx.argument(&argument);
        }

        let raw_result_managed = tx.run();
        let mut raw_result: Vec<Vec<u8>> = raw_result_managed
            .into_vec()
            .iter()
            .map(|buffer| buffer.to_boxed_bytes().into_vec())
            .collect();

        let output_managed = OutputManaged::multi_decode(&mut raw_result).unwrap(); // TODO: no unwrap

        Ok(output_managed.to_native())
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