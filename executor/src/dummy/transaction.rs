use std::mem;

use async_trait::async_trait;
use multiversx_sc::codec::{TopDecodeMulti, TopEncodeMulti};
use multiversx_sc::imports::{Tx, TxScEnv};
use multiversx_sc_scenario::imports::{AddressValue, Bech32Address};
use multiversx_sc_scenario::scenario_model::{ScDeployStep, TypedScDeploy};
use num_bigint::BigUint;

use novax_data::{Address, NativeConvertible};

use crate::base::deploy::DeployExecutor;
use crate::base::transaction::TransactionExecutor;
use crate::call_result::CallResult;
use crate::error::dummy::DummyExecutorError;
use crate::error::executor::ExecutorError;
use crate::utils::transaction::data::{SendableTransaction, SendableTransactionConvertible};
use crate::utils::transaction::token_transfer::TokenTransfer;
use crate::utils::transaction::transfers::get_egld_or_esdt_transfers;

/// A type alias for `DummyExecutor` handling `ScCallStep`.
pub type DummyTransactionExecutor = DummyExecutor<SendableTransaction>;

/// A type alias for `DummyExecutor` handling `ScDeployStep`.
pub type DummyDeployExecutor = DummyExecutor<SendableTransaction>;

/// A structure for capturing transaction details without performing actual blockchain transactions.
/// It is designed for testing scenarios, especially to fetch `SendableTransaction` details from interactions.
pub struct DummyExecutor<Tx: SendableTransactionConvertible> {
    /// Holds the transaction details.
    pub tx: Option<Tx>,
    /// Optionally holds the caller address.
    pub caller: Option<Address>
}

impl<Tx: SendableTransactionConvertible> DummyExecutor<Tx> {
    /// Retrieves the transaction details encapsulated into a `SendableTransaction`.
    pub fn get_transaction_details(&self) -> Result<SendableTransaction, ExecutorError> {
        if let Some(tx) = &self.tx {
            Ok(tx.to_sendable_transaction())
        } else {
            Err(DummyExecutorError::NoTransactionSent.into())
        }
    }
}

impl<Tx: SendableTransactionConvertible> DummyExecutor<Tx> {
    /// Constructs a new `DummyExecutor` instance.
    ///
    /// # Arguments
    ///
    /// * `caller` - An optional reference to the caller address.
    pub fn new(caller: &Option<Address>) -> DummyExecutor<Tx> {
        DummyExecutor {
            tx: None,
            caller: caller.clone()
        }
    }
}

#[async_trait]
impl TransactionExecutor for DummyExecutor<SendableTransaction> {
    /// Captures the smart contract call details.
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
        let from = if let Some(caller) = &self.caller {
            AddressValue::from(caller)
        } else {
            AddressValue::from(Bech32Address::from_bech32_string(to.to_bech32_string()?))
        };

        let payments = get_egld_or_esdt_transfers(
            egld_value,
            esdt_transfers
        )?;

        let mut tx = Tx::new_with_env(TxScEnv::default())
            .from(from)
            .to(Bech32Address::from_bech32_string(to.to_bech32_string()?))
            .gas(gas_limit)
            .egld_or_multi_esdt(payments)
            .raw_call(function);

        for argument in arguments {
            tx = tx.argument(argument);
        }

        let tx = tx.normalize();

        self.tx = Some(tx.to_sendable_transaction());

        /*
        let mut owned_sc_call_step = mem::replace(sc_call_step, ScCallStep::new().into());

        if let Some(caller) = &self.caller {
            owned_sc_call_step = owned_sc_call_step.from(&multiversx_sc::types::Address::from(caller.to_bytes()));
        }

        self.tx = owned_sc_call_step.sc_call_step;

        Ok(())

         */

        let dummy_result = CallResult {
            response: Default::default(),
            result: None,
        };

        Ok(dummy_result)
    }
}

#[async_trait]
impl DeployExecutor for DummyExecutor<ScDeployStep> {
    /// Captures the smart contract deployment details.
    async fn sc_deploy<OriginalResult>(&mut self, sc_deploy_step: &mut TypedScDeploy<OriginalResult>) -> Result<(), ExecutorError>
        where
            OriginalResult: TopEncodeMulti + Send + Sync,
    {
        let mut owned_sc_deploy_step = mem::replace(sc_deploy_step, ScDeployStep::new().into());

        if let Some(caller) = &self.caller {
            owned_sc_deploy_step = owned_sc_deploy_step.from(&multiversx_sc::types::Address::from(caller.to_bytes()));
        }

        self.tx = Some(owned_sc_deploy_step.sc_deploy_step);

        Ok(())
    }

    /// Indicates that deserialization should be skipped as there is no actual execution.
    async fn should_skip_deserialization(&self) -> bool {
        true
    }
}