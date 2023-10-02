use async_trait::async_trait;
use multiversx_sc::codec::{CodecFrom, TopEncodeMulti};
use multiversx_sc_scenario::scenario_model::{ScCallStep, TypedResponse, TypedScDeploy};
use multiversx_sc_snippets::Interactor;
use multiversx_sdk::wallet::Wallet;
use novax_data::Address;

/// A trait defining the interaction interface with the blockchain.
/// This trait abstracts the blockchain interaction, enabling developers to either use the provided `Interactor` struct from the `multiversx-sdk` crate or mock it for testing purposes.
#[async_trait]
pub trait BlockchainInteractor: Send + Sync {

    /// Creates a new instance of a type implementing `BlockchainInteractor`, usually an `Interactor`.
    ///
    /// # Parameters
    ///
    /// * `gateway_url`: A string slice representing the URL of the blockchain gateway.
    ///
    /// # Returns
    ///
    /// * `Self`: A new instance of a type implementing `BlockchainInteractor`.
    async fn new(gateway_url: &str) -> Self;

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
    async fn sc_call<S>(&mut self, sc_call_step: S)
        where
            S: AsMut<ScCallStep> + Send;

    /// Deploys a smart contract on the blockchain and retrieves the result of the deployment.
    ///
    /// # Type Parameters
    ///
    /// * `OriginalResult`: The type representing the expected result of the deployment.
    ///   It should implement `TopEncodeMulti`, `Send`, and `Sync`.
    /// * `RequestedResult`: The type representing the result to be retrieved.
    ///   It should implement `CodecFrom<OriginalResult>`.
    /// * `S`: A type that implements `AsMut<TypedScDeploy<OriginalResult>>` and `Send`,
    ///   representing the deployment step.
    ///
    /// # Parameters
    ///
    /// * `step`: An instance of `S` representing the deployment step.
    ///
    /// # Returns
    ///
    /// * `(Address, TypedResponse<RequestedResult>)`: A tuple containing the blockchain address
    ///   of the deployed smart contract and a `TypedResponse` holding the requested result.
    async fn sc_deploy_get_result<OriginalResult, RequestedResult, S>(&mut self, step: S) -> (Address, TypedResponse<RequestedResult>)
        where
            OriginalResult: TopEncodeMulti + Send + Sync,
            RequestedResult: CodecFrom<OriginalResult>,
            S: AsMut<TypedScDeploy<OriginalResult>> + Send;
}

/// Implementation of the `BlockchainInteractor` trait for the `Interactor` struct from the `multiversx-sdk` crate.
/// This implementation allows for direct interaction with the blockchain via the provided methods.
#[async_trait]
impl BlockchainInteractor for Interactor {

    /// Asynchronously creates a new `Interactor` instance with the specified gateway URL.
    ///
    /// # Parameters
    ///
    /// * `gateway_url`: A string slice representing the URL of the blockchain gateway.
    ///
    /// # Returns
    ///
    /// * `Self`: A new `Interactor` instance.
    async fn new(gateway_url: &str) -> Self {
        Interactor::new(gateway_url).await
    }

    /// Registers a wallet with the `Interactor`, returning the associated blockchain address.
    ///
    /// # Parameters
    ///
    /// * `wallet`: A `Wallet` instance to be registered.
    ///
    /// # Returns
    ///
    /// * `Address`: The blockchain address associated with the registered wallet.
    fn register_wallet(&mut self, wallet: Wallet) -> Address {
        self.register_wallet(wallet).into()
    }

    /// Asynchronously executes a smart contract call on the blockchain.
    ///
    /// # Type Parameters
    ///
    /// * `S`: A type that implements `AsMut<ScCallStep>` and `Send`, representing the smart contract call step.
    ///
    /// # Parameters
    ///
    /// * `sc_call_step`: An instance of `S` representing the smart contract call step.
    async fn sc_call<S>(&mut self, sc_call_step: S) where S: AsMut<ScCallStep> + Send {
        self.sc_call(sc_call_step).await;
    }

    /// Asynchronously deploys a smart contract on the blockchain and retrieves the result of the deployment.
    ///
    /// # Type Parameters
    ///
    /// * `OriginalResult`: The type representing the expected result of the deployment.
    ///   It should implement `TopEncodeMulti`, `Send`, and `Sync`.
    /// * `RequestedResult`: The type representing the result to be retrieved.
    ///   It should implement `CodecFrom<OriginalResult>`.
    /// * `S`: A type that implements `AsMut<TypedScDeploy<OriginalResult>>` and `Send`,
    ///   representing the deployment step.
    ///
    /// # Parameters
    ///
    /// * `step`: An instance of `S` representing the deployment step.
    ///
    /// # Returns
    ///
    /// * `(Address, TypedResponse<RequestedResult>)`: A tuple containing the blockchain address
    ///   of the deployed smart contract and a `TypedResponse` holding the requested result.
    async fn sc_deploy_get_result<OriginalResult, RequestedResult, S>(&mut self, step: S) -> (Address, TypedResponse<RequestedResult>)
        where
            OriginalResult: TopEncodeMulti + Send + Sync,
            RequestedResult: CodecFrom<OriginalResult>,
            S: AsMut<TypedScDeploy<OriginalResult>> + Send
    {
        let result = self.sc_deploy_get_result(step).await;

        (
            result.0.into(),
            result.1
        )
    }
}