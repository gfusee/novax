use std::time::Duration;

use async_trait::async_trait;
use base64::Engine;
use num_bigint::BigUint;

use novax_data::Address;

use crate::error::transaction::TransactionError;
use crate::ExecutorError;
use crate::network::models::address::info::AddressGatewayInfoAccount;
use crate::network::models::network::config::NetworkGatewayConfig;
use crate::network::transaction::models::send_request::TransactionSendRequest;
use crate::network::transaction::models::transaction_on_network::{FINAL_TRANSACTION_STATUS, TransactionOnNetwork};
use crate::network::utils::address::get_address_info;
use crate::network::utils::network::get_network_config;
use crate::network::utils::transaction::{get_transaction_on_network, send_transaction};
use crate::network::utils::wallet::{SignableTransaction, Wallet};
use crate::utils::date::get_current_timestamp::{get_current_timestamp, get_timestamp_of_next_block};

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

    fn get_sender_address(&self) -> Address;
}

#[derive(Clone, Debug)]
pub struct Interactor {
    pub gateway_url: String,
    pub wallet: Wallet,
    pub network_config: NetworkGatewayConfig,
    pub refresh_strategy: TransactionRefreshStrategy,
    pub timeout: Duration
}

#[derive(Clone, Debug)]
pub enum TransactionRefreshStrategy {
    EachBlock,
    EachDuration(Duration)
}

impl Interactor {
    async fn get_account_info(&self) -> Result<AddressGatewayInfoAccount, ExecutorError> {
        let address = self.wallet.get_address();

        Ok(get_address_info(&self.gateway_url, address).await?.account)
    }

    async fn wait_for_execution(&self, tx_hash: &str) -> Result<TransactionOnNetwork, ExecutorError> {
        let end_timestamp = get_current_timestamp()? + self.timeout;

        loop {
            let transaction_on_network = get_transaction_on_network(
                &self.gateway_url,
                tx_hash
            ).await?;

            if FINAL_TRANSACTION_STATUS.contains(&transaction_on_network.transaction.status.as_ref()) {
                return Ok(transaction_on_network)
            }

            let current_timestamp = get_current_timestamp()?;

            if current_timestamp >= end_timestamp {
                return Err(TransactionError::TimeoutWhenRetrievingTransactionOnNetwork.into())
            }

            match self.refresh_strategy {
                TransactionRefreshStrategy::EachBlock => {
                    let timestamp_of_next_block = get_timestamp_of_next_block(current_timestamp)?;
                    tokio::time::sleep(timestamp_of_next_block - current_timestamp).await;
                }
                TransactionRefreshStrategy::EachDuration(duration) => {
                    tokio::time::sleep(duration).await;
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
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
        version: u32,
        options: u32
    ) -> TransactionSendRequest {
        let base64_encoded_data = base64::engine::general_purpose::STANDARD.encode(data);

        let tx_to_sign = SignableTransaction {
            nonce,
            value,
            receiver,
            sender,
            gas_price,
            gas_limit,
            data: Some(base64_encoded_data),
            chain_id,
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
                network_config,
                refresh_strategy: TransactionRefreshStrategy::EachBlock,
                timeout: Duration::from_secs(10)
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
        let nonce = sender_info.nonce;

        let transaction_request = self.get_sendable_transaction(
            nonce,
            value.to_string(),
            to,
            sender_address,
            self.network_config.config.erd_min_gas_price,
            gas_limit,
            data,
            self.network_config.config.erd_chain_id.clone(),
            1,
            0
        );

        let tx_hash = send_transaction(
            &self.gateway_url,
            &transaction_request
        )
            .await?;


        self.wait_for_execution(&tx_hash).await
    }

    fn get_sender_address(&self) -> Address {
        self.wallet.get_address()
    }
}
