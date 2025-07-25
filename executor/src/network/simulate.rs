use std::fmt::{Debug, Formatter};

use async_trait::async_trait;
use base64::Engine;
use multiversx_sc::codec::TopDecodeMulti;
use num_bigint::BigUint;
use tokio::join;

use novax_data::{Address, NativeConvertible};
use novax_request::gateway::client::GatewayClient;

use crate::{ExecutorError, GatewayError, SimulationError, SimulationGatewayRequest, SimulationGatewayResponse, TransactionExecutor, TransactionOnNetwork, TransactionOnNetworkTransaction, TransactionOnNetworkTransactionSmartContractResult};
use crate::call_result::CallResult;
use crate::network::models::simulate::request::SimulationGatewayRequestBody;
use crate::network::utils::address::{get_address_guardian_data, get_address_info};
use crate::network::utils::network::get_network_config;
use crate::utils::transaction::normalization::NormalizationInOut;
use crate::utils::transaction::results::{find_sc_error, find_smart_contract_result};
use crate::utils::transaction::token_transfer::TokenTransfer;

/// Type alias for `BaseSimulationNetworkExecutor` with the `String` type as the generic `Client`.
pub type SimulationNetworkExecutor = BaseSimulationNetworkExecutor<String>;

/// A struct for executing transactions in a simulated blockchain environment.
/// It interacts with a blockchain network for transaction simulation purposes.
pub struct BaseSimulationNetworkExecutor<Client: GatewayClient> {
    /// The client used to interact with the blockchain network gateway for transaction simulations.
    pub client: Client,

    /// The blockchain address of the transaction sender.
    pub sender_address: Address,
}

impl<Client: GatewayClient> BaseSimulationNetworkExecutor<Client> {
    /// Constructs a new `BaseSimulationNetworkExecutor`.
    ///
    /// # Parameters
    /// - `client`: The client for interacting with the blockchain network gateway.
    /// - `sender_address`: The blockchain address that will be used as the sender in the transactions.
    ///
    /// # Returns
    /// A new instance of `BaseSimulationNetworkExecutor`.
    pub fn new(client: Client, sender_address: Address) -> Self {
        Self {
            client,
            sender_address,
        }
    }
}

impl<Client: GatewayClient> BaseSimulationNetworkExecutor<Client> {
    /// Simulates a blockchain transaction and fetches the result.
    ///
    /// # Parameters
    /// - `data`: The transaction data encapsulated in `SimulationGatewayRequest`.
    ///
    /// # Returns
    /// A `Result` containing `SimulationGatewayResponse` on success, or an `ExecutorError` on failure.
    async fn simulate_transaction(&self, data: SimulationGatewayRequest) -> Result<SimulationGatewayResponse, ExecutorError> {
        let sender_address = Address::from_bech32_string(&data.sender)?;

        let (
            address_info,
            address_guardian_data,
            network_config
        ) = join!(
            get_address_info(&self.client, sender_address.clone()),
            get_address_guardian_data(&self.client, sender_address),
            get_network_config(&self.client)
        );

        let address_info = address_info?.account;
        let address_guardian_data = address_guardian_data?;
        let network_config = network_config?.config;
        
        // See https://github.com/multiversx/mx-chain-go/issues/7054
        let (guardian, guardian_signature, version, options) = if address_guardian_data.guardian_data.is_some() {
            let guardian_data = address_guardian_data.guardian_data.unwrap();
            
            if guardian_data.guarded && guardian_data.active_guardian.is_some() {
                let active_guardian = guardian_data.active_guardian.unwrap();

                (
                    Some(active_guardian.address),
                    Some("00".to_string()),
                    Some(2),
                    Some(2)
                )
            } else {
                (None, None, None, None)
            }
        } else {
            (None, None, None, None)
        };

        let body = SimulationGatewayRequestBody {
            nonce: address_info.nonce,
            value: data.value,
            receiver: data.receiver,
            sender: data.sender,
            gas_price: network_config.erd_min_gas_price,
            gas_limit: data.gas_limit,
            data: base64::engine::general_purpose::STANDARD.encode(data.data),
            chain_id: network_config.erd_chain_id,
            version: version.unwrap_or(network_config.erd_min_transaction_version),
            guardian,
            guardian_signature,
            options
        };
        
        let Ok((_, Some(text))) = self.client.with_appended_url("/transaction/cost?checkSignature=false").post(&body).await else {
            return Err(GatewayError::CannotSimulateTransaction.into())
        };

        let Ok(results) = serde_json::from_str(&text) else {
            return Err(GatewayError::CannotParseSimulationResponse.into())
        };

        Ok(results)
    }
}

impl<Client> Clone for BaseSimulationNetworkExecutor<Client>
    where
        Client: GatewayClient + Clone
{
    /// Creates a clone of the `BaseSimulationNetworkExecutor` instance.
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            sender_address: self.sender_address.clone(),
        }
    }
}

impl<Client> Debug for BaseSimulationNetworkExecutor<Client>
    where
        Client: GatewayClient
{
    /// Formats the `BaseSimulationNetworkExecutor` instance for use with the `Debug` trait.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BaseSimulationNetworkExecutor")
            .field("client's url", &self.client.get_gateway_url())
            .field("sender address", &self.sender_address)
            .finish()
    }
}

#[async_trait]
impl<Client: GatewayClient> TransactionExecutor for BaseSimulationNetworkExecutor<Client> {
    /// Executes a smart contract call in a simulated environment.
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
            sender: self.sender_address.to_bech32_string()?,
            receiver: to.to_bech32_string()?,
            function_name,
            arguments,
            egld_value,
            esdt_transfers,
        }.normalize()?;

        let normalized_egld_value = normalized.egld_value.clone();
        let normalized_receiver = normalized.receiver.clone();
        let normalized_sender = normalized.sender.clone();

        let simulation_data = SimulationGatewayRequest {
            value: normalized_egld_value.to_string(),
            receiver: normalized_receiver,
            sender: normalized_sender,
            gas_limit,
            data: normalized.get_transaction_data(),
        };

        let response = self.simulate_transaction(simulation_data).await?;

        let Some(data) = response.data else {
            return Err(SimulationError::ErrorInTx { code: response.code, error: response.error }.into())
        };

        let scrs = data.smart_contract_results
            .into_iter()
            .filter_map(|(hash, result)| {
                let data = result.data?;

                Some(
                    TransactionOnNetworkTransactionSmartContractResult {
                        hash,
                        nonce: result.nonce,
                        data,
                    }
                )
            })
            .collect();

        let fake_tx_on_network = TransactionOnNetwork {
            transaction: TransactionOnNetworkTransaction {
                smart_contract_results: Some(scrs),
                logs: data.logs.clone(),
                ..Default::default()
            }
        };

        let opt_smart_contract_results = find_smart_contract_result(
            &fake_tx_on_network
        )?;

        let mut raw_result = match opt_smart_contract_results {
            None => {
                if let Some(logs) = data.logs.as_ref() {
                    if let Ok(Some(error_log)) = find_sc_error(logs) {
                        return Err(SimulationError::SmartContractExecutionError { // TODO add tests for this
                            status: error_log.status,
                            message: error_log.message
                        }.into());
                    }
                }

                vec![]
            }
            Some(results) => {
                results
            }
        };

        let Ok(output_managed) = OutputManaged::multi_decode(&mut raw_result) else {
            return Err(SimulationError::CannotDecodeSmartContractResult.into())
        };

        let mut response = TransactionOnNetwork::default();
        response.transaction.status = "success".to_string();

        let call_result = CallResult {
            response,
            result: Some(output_managed.to_native()),
        };

        Ok(call_result)
    }
}