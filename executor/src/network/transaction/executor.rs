use std::fmt::{Debug, Formatter};
use std::time::Duration;

use async_trait::async_trait;
use multiversx_sc::codec::TopDecodeMulti;
use multiversx_sc::imports::CodeMetadata;
use num_bigint::BigUint;

use novax_data::{Address, NativeConvertible};

use crate::base::deploy::DeployExecutor;
use crate::base::transaction::TransactionExecutor;
use crate::call_result::CallResult;
use crate::error::executor::ExecutorError;
use crate::error::transaction::TransactionError;
use crate::network::transaction::interactor::{BlockchainInteractor, Interactor, TransactionRefreshStrategy};
use crate::network::utils::wallet::Wallet;
use crate::utils::transaction::deploy::get_deploy_call_input;
use crate::utils::transaction::normalization::NormalizationInOut;
use crate::utils::transaction::results::{find_sc_deploy_event, find_smart_contract_result};
use crate::utils::transaction::token_transfer::TokenTransfer;

/// Alias for the `BaseTransactionNetworkExecutor` struct, parameterized with the `Interactor` type.
pub type NetworkExecutor = BaseTransactionNetworkExecutor<Interactor>;

/// A struct representing the executor for handling transactions in a real blockchain environment.
///
/// This executor is designed to interact with a blockchain network via a specified gateway URL and a wallet
/// for signing transactions. It is parameterized by a type `Interactor` that encapsulates the blockchain interaction logic.
pub struct BaseTransactionNetworkExecutor<Interactor: BlockchainInteractor> {
    interactor: Interactor
}

impl BaseTransactionNetworkExecutor<Interactor> {
    pub fn set_refresh_strategy(&mut self, strategy: TransactionRefreshStrategy) {
        self.interactor.refresh_strategy = strategy;
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.interactor.timeout = timeout;
    }
}

/// Custom implementation of `Clone` for `BaseTransactionNetworkExecutor`, when `Interactor` is `Clone`.
impl<Interactor> Clone for BaseTransactionNetworkExecutor<Interactor>
    where
        Interactor: BlockchainInteractor + Clone
{
    fn clone(&self) -> Self {
        Self {
            interactor: self.interactor.clone()
        }
    }
}

/// Custom implementation of `Debug` for `BaseTransactionNetworkExecutor`, when `Interactor` is `Debug`.
///
/// The implementation is basic, it prefixes "BaseTransactionNetworkExecutor" before the `Interactor`'s debug implementation.
impl<Interactor> Debug for BaseTransactionNetworkExecutor<Interactor>
    where
        Interactor: BlockchainInteractor + Debug
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "BaseTransactionNetworkExecutor - ")?;
        self.interactor.fmt(f)
    }
}

impl<Interactor: BlockchainInteractor> BaseTransactionNetworkExecutor<Interactor> {
    /// Creates a new instance of `BaseTransactionNetworkExecutor`.
    ///
    /// This function is async because the Interactor may perform some requests, such as retrieving the network configuration.
    /// Those async operations might fail, thus the Result return type.
    pub async fn new(gateway_url: String, wallet: Wallet) -> Result<Self, ExecutorError> {
        let interactor = Interactor::new(
            gateway_url,
            wallet
        ).await?;

        Ok(
            BaseTransactionNetworkExecutor {
                interactor
            }
        )
    }
}

#[async_trait]
impl<Interactor: BlockchainInteractor> TransactionExecutor for BaseTransactionNetworkExecutor<Interactor> {
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

        let normalized = NormalizationInOut {
            sender: self.interactor.get_sender_address().to_bech32_string()?,
            receiver: to.to_bech32_string()?,
            function_name,
            arguments,
            egld_value,
            esdt_transfers,
        }.normalize()?;

        let receiver = normalized.receiver.clone();
        let egld_value = normalized.egld_value.clone();
        let transaction_data = normalized.get_transaction_data();

        let result = self.interactor.sc_call(
            receiver,
            egld_value,
            transaction_data,
            gas_limit,
        )
            .await?;

        let Some(mut sc_result) = find_smart_contract_result(
            &result.transaction.smart_contract_results,
            result.transaction.logs.as_ref()
        )? else {
            return Err(TransactionError::NoSmartContractResult.into())
        };

        let managed_result = OutputManaged::multi_decode(&mut sc_result)
            .map_err(|_| TransactionError::CannotDecodeSmartContractResult)?;

        let native_result = managed_result.to_native();

        let call_result = CallResult {
            response: result,
            result: Some(native_result),
        };

        Ok(call_result)
    }
}

/// Implementation of the `DeployExecutor` trait for the `BaseTransactionNetworkExecutor` struct.
/// This implementation enables the deployment of smart contracts on the blockchain
/// using a specified blockchain interactor.
#[async_trait]
impl<Interactor: BlockchainInteractor> DeployExecutor for BaseTransactionNetworkExecutor<Interactor> {

    /// Asynchronously deploys a smart contract to the blockchain.
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

        let Some(logs) = deploy_result.response.transaction.logs.as_ref() else {
            return Err(TransactionError::NoSCDeployLogInTheResponse.into())
        };

        let Some(sc_deploy_event) = find_sc_deploy_event(&logs.events) else {
            return Err(TransactionError::NoSCDeployLogInTheResponse.into())
        };

        let deployed_address = Address::from_bech32_string(&sc_deploy_event.address)?;

        Ok((deployed_address, deploy_result))
    }
}