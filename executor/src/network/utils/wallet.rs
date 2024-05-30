use std::fmt::{Debug, Formatter};
use multiversx_sdk::crypto::private_key::{PRIVATE_KEY_LENGTH, PrivateKey};
use multiversx_sdk::crypto::public_key::PublicKey;
use serde::Serialize;
use serde_json::json;
use sha3::{Digest, Keccak256};
use novax_data::Address;
use crate::error::wallet::WalletError;
use crate::ExecutorError;
use crate::network::transaction::models::send_request::TransactionSendRequest;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignableTransaction {
    pub nonce: u64,
    pub value: String,
    pub receiver: String,
    pub sender: String,
    pub gas_price: u64,
    pub gas_limit: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(rename = "chainID")]
    pub chain_id: String,
    pub version: u32,
    #[serde(skip_serializing_if = "is_zero")]
    pub options: u32,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(num: &u32) -> bool {
    *num == 0
}

impl SignableTransaction {
    pub fn into_sendable_transaction(self, wallet: &Wallet) -> TransactionSendRequest {
        let signature = wallet.sign_transaction(&self);

        TransactionSendRequest {
            nonce: self.nonce,
            value: self.value,
            receiver: self.receiver,
            sender: self.sender,
            gas_price: self.gas_price,
            gas_limit: self.gas_limit,
            data: self.data,
            signature,
            chain_id: self.chain_id,
            version: self.version,
            options: self.options,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Wallet(PrivateKey);

impl Wallet {
    pub fn from_private_key(private_key: &str) -> Result<Wallet, ExecutorError> {
        let private_key = PrivateKey::from_hex_str(private_key)
            .map_err(|_| WalletError::InvalidPrivateKey)?;

        Ok(Wallet(private_key))
    }

    pub fn from_pem_file(file_path: &str) -> Result<Self, ExecutorError> {
        let contents = std::fs::read_to_string(file_path)
            .map_err(|_| WalletError::InvalidPemFile)?;

        Self::from_pem_file_contents(contents)
    }

    pub fn from_pem_file_contents(contents: String) -> Result<Self, ExecutorError> {
        let x = pem::parse(contents)
            .map_err(|_| WalletError::InvalidPemFile)?;
        let x = x.contents()[..PRIVATE_KEY_LENGTH].to_vec();
        let priv_key_str = std::str::from_utf8(x.as_slice())
            .map_err(|_| WalletError::InvalidPemFile)?;
        let pri_key = PrivateKey::from_hex_str(priv_key_str)
            .map_err(|_| WalletError::InvalidPemFile)?;
        Ok(Self(pri_key))
    }

    pub fn get_address(&self) -> Address {
        let public_key = PublicKey::from(&self.0);
        Address::from(multiversx_sdk::data::address::Address::from(&public_key))
    }

    pub fn sign_transaction(&self, transaction: &SignableTransaction) -> String {
        let mut tx_bytes = json!(transaction).to_string().as_bytes().to_vec();

        let should_sign_on_tx_hash = transaction.version >= 2 && transaction.options & 1 > 0;
        if should_sign_on_tx_hash {
            let mut h = Keccak256::new();
            h.update(tx_bytes);
            tx_bytes = h.finalize().as_slice().to_vec();
        }

        hex::encode(self.0.sign(tx_bytes))
    }
}

impl Debug for Wallet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_address().to_bech32_string().unwrap())
    }
}