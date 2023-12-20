use std::fmt::{Debug, Formatter};
use async_trait::async_trait;
use base64::Engine;
use multiversx_sc_scenario::scenario_model::{TxResponse, TypedScCall};
use multiversx_sdk::data::transaction::ApiSmartContractResult;
use multiversx_sdk::data::vm::CallType;
use tokio::join;
use novax_data::Address;
use novax_request::gateway::client::GatewayClient;
use crate::{ExecutorError, GatewayError, SendableTransactionConvertible, SimulationError, SimulationGatewayRequest, SimulationGatewayResponse, TransactionExecutor};
use crate::network::models::simulate::request::SimulationGatewayRequestBody;
use crate::network::utils::address::get_address_info;
use crate::network::utils::network::get_network_config;

pub type SimulationNetworkExecutor = BaseSimulationNetworkExecutor<String>;

pub struct BaseSimulationNetworkExecutor<Client: GatewayClient> {
    /// The URL of the blockchain network gateway through which transactions will be sent.
    pub client: Client,
    pub sender_address: Address,
}

impl<Client: GatewayClient> BaseSimulationNetworkExecutor<Client> {
    pub fn new(client: Client, sender_address: Address) -> Self {
        Self {
            client,
            sender_address,
        }
    }
}

impl<Client: GatewayClient> BaseSimulationNetworkExecutor<Client> {
    async fn simulate_transaction(&self, data: SimulationGatewayRequest) -> Result<SimulationGatewayResponse, ExecutorError> {
        let sender_address = Address::from_bech32_string(&data.sender)?;

        let (
            address_info,
            network_config
        ) = join!(
            get_address_info(&self.client, sender_address),
            get_network_config(&self.client)
        );

        let address_info = address_info?.account;
        let network_config = network_config?.config;

        let body = SimulationGatewayRequestBody {
            nonce: address_info.nonce,
            value: data.value,
            receiver: data.receiver,
            sender: data.sender,
            gas_price: network_config.erd_min_gas_price,
            gas_limit: data.gas_limit,
            data: base64::engine::general_purpose::STANDARD.encode(data.data),
            chain_id: network_config.erd_chain_id,
            version: network_config.erd_min_transaction_version,
        };

        let Ok(response) = self.client.with_appended_url("/transaction/cost").post(&body).await else {
            return Err(GatewayError::CannotSimulateTransaction.into())
        };

        let Ok(text) = response.text().await else {
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BaseSimulationNetworkExecutor")
            .field("client's url", &self.client.get_gateway_url())
            .field("sender address", &self.sender_address)
            .finish()
    }
}

#[async_trait]
impl<Client: GatewayClient> TransactionExecutor for BaseSimulationNetworkExecutor<Client> {
    async fn sc_call<OriginalResult: Send>(&mut self, sc_call_step: &mut TypedScCall<OriginalResult>) -> Result<(), ExecutorError> {
        let sendable_transaction = sc_call_step.to_sendable_transaction();

        let simulation_data = SimulationGatewayRequest {
            value: sendable_transaction.egld_value.to_string(),
            receiver: sendable_transaction.receiver,
            sender: self.sender_address.to_bech32_string()?,
            gas_limit: sendable_transaction.gas_limit,
            data: sendable_transaction.data,
        };

        let response = self.simulate_transaction(simulation_data).await?;

        let Some(data) = response.data else {
            return Err(SimulationError::ErrorInTx { code: response.code, error: response.error }.into())
        };

        let scrs = data.smart_contract_results
            .into_iter()
            .map(|(hash, result)| {
                ApiSmartContractResult {
                    hash,
                    nonce: result.nonce,
                    value: result.value,
                    receiver: multiversx_sdk::data::address::Address::from_bech32_string(&result.receiver).unwrap(),
                    sender: multiversx_sdk::data::address::Address::from_bech32_string(&result.sender).unwrap(),
                    data: result.data,
                    prev_tx_hash: "".to_string(),
                    original_tx_hash: "".to_string(),
                    gas_limit: 0,
                    gas_price: 0,
                    call_type: CallType::DirectCall,
                    relayer_address: None,
                    relayed_value: None,
                    code: None,
                    code_metadata: None,
                    return_message: None,
                    original_sender: None,
                }
            })
            .collect();

        let tx_response = TxResponse {
            out: vec![],
            new_deployed_address: None,
            new_issued_token_identifier: None,
            tx_error: Default::default(),
            logs: vec![],
            gas: data.tx_gas_units,
            refund: 0,
            api_scrs: scrs,
            api_logs: None,
        };

        sc_call_step.sc_call_step.save_response(tx_response);

        Ok(())
    }

    async fn should_skip_deserialization(&self) -> bool {
        false
    }
}