use std::ops::Deref;
use std::sync::Arc;

use async_trait::async_trait;
use multiversx_sc::codec::TopDecodeMulti;
use multiversx_sc::imports::{CodeMetadata, ReturnsNewAddress};
use multiversx_sc::types::ReturnsRawResult;
use multiversx_sc_scenario::imports::{Bech32Address, BytesValue};
use multiversx_sc_scenario::ScenarioTxRun;
use num_bigint::BigUint;
use tokio::sync::Mutex;

use novax_data::Address;
use novax_data::NativeConvertible;

use crate::base::deploy::DeployExecutor;
use crate::base::query::QueryExecutor;
use crate::base::transaction::TransactionExecutor;
use crate::call_result::CallResult;
use crate::error::executor::ExecutorError;
use crate::error::mock_deploy::MockDeployError;
use crate::error::mock_transaction::MockTransactionError;
use crate::error::transaction::TransactionError;
use crate::{ScenarioWorld, TransactionOnNetwork};
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
    /// This method updates the world state accordingly, all within a controlled, mocked environment.
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

        let Ok(output_managed) = OutputManaged::multi_decode(&mut raw_result) else {
            return Err(TransactionError::CannotDecodeSmartContractResult.into())
        };

        let mut response = TransactionOnNetwork::default();
        response.transaction.status = "successful".to_string();

        let call_result = CallResult {
            response,
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

        let Ok(output_managed) = OutputManaged::multi_decode(&mut raw_result) else {
            return Err(TransactionError::CannotDecodeSmartContractResult.into())
        };

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
    async fn execute<OutputManaged>(
        &self,
        to: &Address,
        function: String,
        arguments: Vec<Vec<u8>>,
        _egld_value: BigUint,
        _esdt_transfers: Vec<TokenTransfer>
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

        let Ok(output_managed) = OutputManaged::multi_decode(&mut raw_result) else {
            return Err(TransactionError::CannotDecodeSmartContractResult.into())
        };

        Ok(output_managed.to_native())
    }
}