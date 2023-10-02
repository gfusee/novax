use std::marker::PhantomData;
use std::mem;
use async_trait::async_trait;
use multiversx_sc::codec::multi_types::IgnoreValue;
use multiversx_sc::codec::TopEncodeMulti;
use multiversx_sc_scenario::scenario_model::{ScCallStep, ScDeployStep, TypedScCall, TypedScDeploy};
use multiversx_sc_snippets::Interactor;
use multiversx_sdk::wallet::Wallet;
use crate::base::deploy::DeployExecutor;
use crate::base::transaction::TransactionExecutor;
use crate::error::executor::ExecutorError;
use crate::network::interactor::BlockchainInteractor;

/// Alias for the `BaseTransactionNetworkExecutor` struct, parameterized with the `Interactor` type.
pub type NetworkExecutor = BaseTransactionNetworkExecutor<Interactor>;

/// A struct representing the executor for handling transactions in a real blockchain environment.
///
/// This executor is designed to interact with a blockchain network via a specified gateway URL and a wallet
/// for signing transactions. It is parameterized by a type `Interactor` that encapsulates the blockchain interaction logic.
pub struct BaseTransactionNetworkExecutor<Interactor: BlockchainInteractor> {
    /// The URL of the blockchain network gateway through which transactions will be sent.
    pub gateway_url: String,
    /// The wallet used for signing transactions before they are sent to the blockchain network.
    pub wallet: Wallet,
    /// Phantom data to allow the generic parameter `Interactor`.
    /// This field does not occupy any space in memory.
    _phantom_data: PhantomData<Interactor>,
}

impl<Interactor: BlockchainInteractor> BaseTransactionNetworkExecutor<Interactor> {
    /// Creates a new instance of `BaseTransactionNetworkExecutor`.
    ///
    /// # Parameters
    /// - `gateway_url`: The URL of the blockchain network gateway.
    /// - `wallet`: A reference to the wallet used for signing transactions.
    ///
    /// # Returns
    /// A new `BaseTransactionNetworkExecutor` instance.
    pub fn new(gateway_url: &str, wallet: &Wallet) -> Self {
        BaseTransactionNetworkExecutor {
            gateway_url: gateway_url.to_string(),
            wallet: *wallet,
            _phantom_data: PhantomData,
        }
    }
}

#[async_trait]
impl<Interactor: BlockchainInteractor> TransactionExecutor for BaseTransactionNetworkExecutor<Interactor> {
    /// Executes a smart contract call on the blockchain.
    ///
    /// # Parameters
    /// - `sc_call_step`: A mutable reference to the smart contract call step.
    ///
    /// # Type Parameters
    /// - `OriginalResult`: The type of the result expected from the smart contract call. Must implement the `Send` trait.
    ///
    /// # Returns
    /// - A `Result` with an empty `Ok(())` value if the call is successful, or an `Err(ExecutorError)` if the call fails.
    async fn sc_call<OriginalResult: Send>(&mut self, sc_call_step: &mut TypedScCall<OriginalResult>) -> Result<(), ExecutorError> {
        let owned_sc_call_step = mem::replace(sc_call_step, ScCallStep::new().into());
        let mut interactor = Interactor::new(&self.gateway_url).await;
        let sender_address = interactor.register_wallet(self.wallet);
        *sc_call_step = owned_sc_call_step.from(&multiversx_sc::types::Address::from(sender_address.to_bytes()));

        interactor.sc_call(sc_call_step).await;

        Ok(())
    }

    /// Indicates whether deserialization should be skipped during smart contract call execution.
    ///
    /// In the context of a real blockchain environment, deserialization is not skipped,
    /// hence this method returns `false`.
    ///
    /// # Returns
    /// - A boolean value `false`, indicating that deserialization should not be skipped.
    async fn should_skip_deserialization(&self) -> bool {
        false
    }
}

/// Implementation of the `DeployExecutor` trait for the `BaseTransactionNetworkExecutor` struct.
/// This implementation enables the deployment of smart contracts on the blockchain
/// using a specified blockchain interactor.
#[async_trait]
impl<Interactor: BlockchainInteractor> DeployExecutor for BaseTransactionNetworkExecutor<Interactor> {

    /// Asynchronously deploys a smart contract to the blockchain.
    ///
    /// # Type Parameters
    ///
    /// * `OriginalResult`: Represents the result type expected from the smart contract deployment.
    ///    This type must implement `TopEncodeMulti`, `Send`, and `Sync`.
    /// * `S`: Represents the type encapsulating the smart contract deployment step.
    ///    This type must implement `AsMut<TypedScDeploy<OriginalResult>>` and `Send`.
    ///
    /// # Parameters
    ///
    /// * `sc_deploy_step`: A mutable reference to the smart contract deployment step to be executed.
    ///
    /// # Returns
    ///
    /// A `Result` with an empty `Ok(())` value indicating success, or an `Err(ExecutorError)` indicating failure.
    async fn sc_deploy<OriginalResult, S>(&mut self, mut sc_deploy_step: S) -> Result<(), ExecutorError>
        where
            OriginalResult: TopEncodeMulti + Send + Sync,
            S: AsMut<TypedScDeploy<OriginalResult>> + Send
    {
        let sc_deploy_step = sc_deploy_step.as_mut();
        let owned_sc_deploy_step = mem::replace(sc_deploy_step, ScDeployStep::new().into());
        let mut interactor = Interactor::new(&self.gateway_url).await;
        let sender_address = interactor.register_wallet(self.wallet);
        *sc_deploy_step = owned_sc_deploy_step.from(&multiversx_sc::types::Address::from(sender_address.to_bytes()));

        interactor.sc_deploy_get_result::<OriginalResult, IgnoreValue, _>(sc_deploy_step).await;

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
