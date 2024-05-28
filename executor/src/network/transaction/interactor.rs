use std::time::Duration;
use async_trait::async_trait;
use multiversx_sc_scenario::scenario_model::ScDeployStep;
use num_bigint::BigUint;
use reqwest::Client;

use novax_data::{Address, NativeConvertible};
use crate::call_result::CallResult;
use crate::error::transaction::TransactionError;

use crate::{ExecutorError, TopDecodeMulti};
use crate::network::models::address::info::AddressGatewayInfoAccount;
use crate::network::models::network::config::NetworkGatewayConfig;
use crate::network::transaction::models::send_request::TransactionSendRequest;
use crate::network::transaction::models::send_response::TransactionSendResponse;
use crate::network::transaction::models::transaction_on_network::TransactionOnNetwork;
use crate::network::utils::address::get_address_info;
use crate::network::utils::network::get_network_config;
use crate::network::utils::transaction::{get_transaction_on_network, send_transaction};
use crate::network::utils::wallet::{SignableTransaction, Wallet};

#[async_trait]
pub trait BlockchainInteractor: Sized + Send + Sync {
    async fn new(gateway_url: String, wallet: Wallet) -> Result<Self, ExecutorError>;

    async fn sc_call(
        &mut self,
        to: String,
        value: BigUint,
        data: String,
        gas_limit: u64
    ) -> Result<TransactionOnNetwork, ExecutorError>;
}

pub struct Interactor {
    pub gateway_url: String,
    pub wallet: Wallet,
    pub network_config: NetworkGatewayConfig
}

pub enum TransactionRefreshStrategy {
    EachBlock,
    EachDuration(Duration)
}

impl Interactor {
    async fn get_account_info(&self) -> Result<AddressGatewayInfoAccount, ExecutorError> {
        let address = Address::from(self.wallet.get_address());

        Ok(get_address_info(&self.gateway_url, address).await?.account)
    }

    async fn wait_for_execution(&self, tx_hash: &str) -> Result<TransactionOnNetwork, ExecutorError> {
        loop {
            let transaction_on_network = get_transaction_on_network(
                &self.gateway_url,
                tx_hash
            ).await?;

            if transaction_on_network.transaction.status == "executed" {
                return Ok(transaction_on_network)
            }

            tokio::time::sleep(Duration::from_secs(1)).await; // TODO
        }

        // TODO: add timeout
    }
    
    fn get_sendable_transaction(
        &self,
        nonce: u64,
        value: String,
        receiver: String,
        sender: String,
        gas_price: u64,
        gas_limit: u64,
        data: String,
        chain_id: String,
        version: u64,
        options: u64
    ) -> TransactionSendRequest {
        let tx_to_sign = SignableTransaction {
            nonce,
            value: value.clone(),
            receiver: receiver.clone(),
            sender: sender.clone(),
            gas_price,
            gas_limit,
            data: data.clone(),
            chain_id: chain_id.clone(),
            version,
            options,
        };

        tx_to_sign.into_sendable_transaction(&self.wallet)
    }
}

#[async_trait]
impl BlockchainInteractor for Interactor {
    async fn new(
        gateway_url: String,
        wallet: Wallet
    ) -> Result<Self, ExecutorError> {
        let network_config = get_network_config(&gateway_url).await?;

        Ok(
            Self {
                gateway_url,
                wallet,
                network_config
            }
        )
    }

    async fn sc_call(
        &mut self,
        to: String,
        value: BigUint,
        data: String,
        gas_limit: u64
    ) -> Result<TransactionOnNetwork, ExecutorError> {
        let sender_info = self.get_account_info().await?;
        let sender_address = sender_info.address;
        let nonce = sender_info.nonce; // TODO: +1?

        let transaction_request = self.get_sendable_transaction(
            nonce,
            value.to_string(),
            to,
            sender_address,
            self.network_config.config.erd_min_gas_price,
            gas_limit,
            data,
            self.network_config.config.erd_chain_id.clone(),
            1, // TODO: what's this?
            0 // TODO: what's this?
        );

        let tx_hash = send_transaction(
            &self.gateway_url,
            &transaction_request
        )
            .await?;


        self.wait_for_execution(&tx_hash).await
    }
}

fn encode_code_bytes(bytes: &[u8]) -> String {
    hex::encode(bytes)
}