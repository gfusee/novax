use async_trait::async_trait;
use multiversx_sc::imports::EgldOrMultiEsdtPayment;
use multiversx_sc_scenario::api::StaticApi;
use multiversx_sc_scenario::scenario_model::ScDeployStep;
use multiversx_sdk::wallet::Wallet;

use novax_data::Address;

use crate::call_result::CallResult;
use crate::ExecutorError;
use crate::utils::transaction::transfers::EgldOrMultiEsdtTransfers;

/// A trait defining the interaction interface with the blockchain.
/// This trait abstracts the blockchain interaction, enabling developers to either use the provided `Interactor` struct from the `multiversx-sdk` crate or mock it for testing purposes.
#[async_trait]
pub trait BlockchainInteractor: Send + Sync {

    /// Creates a new instance of a type implementing `BlockchainInteractor`, usually an `Interactor`.
    ///
    /// # Parameters
    ///
    /// * `gateway_url`: A string representing the URL of the blockchain gateway.
    ///
    /// # Returns
    ///
    /// * `Self`: A new instance of a type implementing `BlockchainInteractor`.
    async fn new(gateway_url: String) -> Self;

    /// Registers a wallet with the blockchain interactor, returning the associated blockchain address.
    ///
    /// # Parameters
    ///
    /// * `wallet`: A `Wallet` instance to be registered.
    ///
    /// # Returns
    ///
    /// * `Address`: The blockchain address associated with the registered wallet.
    fn register_wallet(&mut self, wallet: Wallet) -> Address;

    /// Executes a smart contract call on the blockchain.
    ///
    /// # Type Parameters
    ///
    /// * `S`: A type that implements `AsMut<ScCallStep>` and `Send`, representing the smart contract call step.
    ///
    /// # Parameters
    ///
    /// * `sc_call_step`: An instance of `S` representing the smart contract call step.
    async fn sc_call(
        &mut self,
        from: &Address,
        to: &Address,
        function: &str,
        arguments: &[Vec<u8>],
        gas_limit: u64,
        payment: EgldOrMultiEsdtTransfers
    ) -> Result<Vec<Vec<u8>>, ExecutorError>;

    /// Deploys a smart contract on the blockchain.
    ///
    /// The `sc_deploy` method takes a `sc_deploy_step` parameter that encapsulates the information required
    /// for deploying a smart contract. The method is asynchronous and requires the [`tokio`] runtime, ensuring non-blocking
    /// operation and concurrency where needed.
    ///
    /// # Type Parameters
    /// - `S`: A type that implements [`AsMut<ScDeployStep>`] trait, allowing for a mutable reference to an [`ScDeployStep`] instance to be obtained.
    ///
    /// # Parameters
    /// - `&mut self`: A mutable reference to the current [`BlockchainInteractor`] instance.
    /// - `sc_deploy_step`: The smart contract deployment step encapsulating the necessary information for deployment.
    ///
    /// # Returns
    /// The method returns a [`Result`] indicating the success or failure of the operation. Successful operations
    /// will return `Ok(())` while failures will return `Err(BlockchainInteractorError)`.
    ///
    /// # Errors
    /// Any errors that occur during the execution of this method will be encapsulated in a [`BlockchainInteractorError`] and returned.
    async fn sc_deploy<S>(&mut self, sc_deploy_step: S)
        where
            S: AsMut<ScDeployStep> + Send;
}

pub struct Interactor {
    pub gateway_url: String,
    pub wallet: Option<Wallet>
}

#[async_trait]
impl BlockchainInteractor for Interactor {
    async fn new(gateway_url: String) -> Self {
        Self {
            gateway_url,
            wallet: None,
        }
    }

    fn register_wallet(&mut self, wallet: Wallet) -> Address {
        let address = Address::from(wallet.address());
        self.wallet = Some(wallet);

        address
    }

    async fn sc_call(
        &mut self,
        from: &Address,
        to: &Address,
        function: &str,
        arguments: &[Vec<u8>],
        gas_limit: u64,
        payment: EgldOrMultiEsdtTransfers
    ) -> Result<Vec<Vec<u8>>, ExecutorError> {
        todo!()
    }

    async fn sc_deploy<S>(&mut self, sc_deploy_step: S) where S: AsMut<ScDeployStep> + Send {
        todo!()
    }
}