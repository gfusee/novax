use async_trait::async_trait;
use multiversx_sc::codec::TopDecodeMulti;
use multiversx_sc::imports::CodeMetadata;
use num_bigint::BigUint;

use novax_data::{Address, NativeConvertible};

use crate::base::deploy::DeployExecutor;
use crate::base::transaction::TransactionExecutor;
use crate::call_result::CallResult;
use crate::error::dummy::DummyExecutorError;
use crate::error::executor::ExecutorError;
use crate::utils::transaction::data::{SendableTransaction, SendableTransactionConvertible};
use crate::utils::transaction::deploy::get_deploy_call_input;
use crate::utils::transaction::normalization::NormalizationInOut;
use crate::utils::transaction::token_transfer::TokenTransfer;

/// A type alias for `DummyExecutor` handling `SendableTransaction`.
pub type DummyTransactionExecutor = DummyExecutor<SendableTransaction>;

/// A type alias for `DummyExecutor` handling `SendableTransaction`.
pub type DummyDeployExecutor = DummyExecutor<SendableTransaction>;

/// A structure for capturing transaction details without performing actual blockchain transactions.
/// It is designed for testing scenarios, especially to fetch `SendableTransaction` details from interactions.
pub struct DummyExecutor<Tx: SendableTransactionConvertible> {
    /// Holds the transaction details.
    pub tx: Option<Tx>,
    /// Optionally holds the caller address.
    pub caller: Option<Address>
}

impl<Tx: SendableTransactionConvertible + Clone> DummyExecutor<Tx> {
    /// Retrieves the transaction details encapsulated into a `SendableTransaction`.
    pub fn get_transaction_details(&self) -> Result<SendableTransaction, ExecutorError> {
        if let Some(tx) = self.tx.clone() {
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
        arguments: Vec<Vec<u8>>,
        gas_limit: u64,
        egld_value: BigUint,
        esdt_transfers: Vec<TokenTransfer>
    ) -> Result<CallResult<OutputManaged::Native>, ExecutorError>
        where
            OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
    {
        let function_name = if function.is_empty() {
            None
        } else {
            Some(function)
        };

        let to = to.to_bech32_string()?;

        let normalized = NormalizationInOut {
            sender: self.caller.clone().map(|address| address.to_bech32_string()).unwrap_or_else(|| Ok(to.clone()))?,
            receiver: to,
            function_name,
            arguments,
            egld_value,
            esdt_transfers,
        }.normalize()?;

        self.tx = Some(normalized.into_sendable_transaction(gas_limit));

        let dummy_result = CallResult {
            response: Default::default(),
            result: None,
        };

        Ok(dummy_result)
    }
}

#[async_trait]
impl DeployExecutor for DummyExecutor<SendableTransaction> {
    /// Captures the smart contract deployment details.
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
        let deploy_call_input = get_deploy_call_input(
            bytes,
            code_metadata,
            egld_value,
            arguments,
            gas_limit
        );

        let deploy_result = self.sc_call::<OutputManaged>(
            &deploy_call_input.to,
            deploy_call_input.function,
            deploy_call_input.arguments,
            deploy_call_input.gas_limit,
            deploy_call_input.egld_value,
            deploy_call_input.esdt_transfers
        )
            .await?;

        Ok((Address::default(), deploy_result))
    }
}