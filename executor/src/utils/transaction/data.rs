use multiversx_sc::api::CallTypeApi;
use multiversx_sc::types::ContractCallWithEgld;
use multiversx_sc_scenario::scenario_model::{ScCallStep, ScDeployStep, TxCall, TxDeploy, TypedScCall, TypedScDeploy};
use multiversx_sdk::data::address::Address;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

/// Represents a transaction that is ready to be sent to the blockchain.
///
/// This structure contains the necessary information for the frontend to send a transaction
/// once the user has their wallet connected.
#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub struct SendableTransaction {
    /// The receiver's address as a string.
    pub receiver: String,
    /// The amount of EGLD to be sent along with the transaction.
    pub egld_value: BigUint,
    /// The gas limit for the transaction.
    pub gas_limit: u64,
    /// The data payload for the transaction as a string.
    pub data: String
}

/// A trait for converting a type into a [`SendableTransaction`].
///
/// This trait is implemented by various transaction and contract call representations
/// to convert them into a `SendableTransaction`, which is a more frontend-friendly format.
pub trait SendableTransactionConvertible {
    /// Converts the current instance into a [`SendableTransaction`].
    fn to_sendable_transaction(&self) -> SendableTransaction;
}

// Implementations of `SendableTransactionConvertible` for various types, enabling them to be converted into `SendableTransaction`s.

impl<SA, T> SendableTransactionConvertible for ContractCallWithEgld<SA, T>
    where
        SA: CallTypeApi + 'static
{
    fn to_sendable_transaction(&self) -> SendableTransaction {
        SendableTransaction {
            receiver: Address::from_bytes(self.basic.to.to_byte_array()).to_bech32_string().unwrap(),
            egld_value: self.egld_payment.to_alloc(),
            gas_limit: self.basic.explicit_gas_limit,
            data: contract_call_to_tx_data(self),
        }
    }
}

impl SendableTransactionConvertible for TxCall {
    fn to_sendable_transaction(&self) -> SendableTransaction {
        self.to_contract_call().to_sendable_transaction()
    }
}

impl SendableTransactionConvertible for ScCallStep {
    fn to_sendable_transaction(&self) -> SendableTransaction {
        self.tx.to_sendable_transaction()
    }
}

impl<T> SendableTransactionConvertible for TypedScCall<T> {
    fn to_sendable_transaction(&self) -> SendableTransaction {
        self.sc_call_step.to_sendable_transaction()
    }
}

impl SendableTransactionConvertible for TxDeploy {
    fn to_sendable_transaction(&self) -> SendableTransaction {
        let mut call = TxCall {
            from: self.from.clone(),
            to: Default::default(),
            egld_value: self.egld_value.clone(),
            esdt_value: Default::default(),
            function: Default::default(),
            arguments: vec![],
            gas_limit: self.gas_limit.clone(),
            gas_price: self.gas_price.clone(),
        }.to_sendable_transaction();

        call.data = self.to_tx_data();

        call
    }
}

impl SendableTransactionConvertible for ScDeployStep {
    fn to_sendable_transaction(&self) -> SendableTransaction {
        self.tx.to_sendable_transaction()
    }
}

impl<T> SendableTransactionConvertible for TypedScDeploy<T> {
    fn to_sendable_transaction(&self) -> SendableTransaction {
        self.sc_deploy_step.to_sendable_transaction()
    }
}

/// Converts a `ContractCallWithEgld` instance into a transaction data string.
///
/// This function is almost a duplicate of a private function from `mx-sdk-rs`.
/// It generates a string representation of a contract call, suitable for use as the data payload in a transaction.
///
/// # TODO:
/// Remove this function if the one in `mx-sdk-rs` becomes public.
fn contract_call_to_tx_data<SA, T>(contract_call: &ContractCallWithEgld<SA, T>) -> String
    where
        SA: CallTypeApi + 'static
{
    let mut result = String::from_utf8(
        contract_call
            .basic
            .endpoint_name
            .to_boxed_bytes()
            .into_vec(),
    )
        .unwrap();

    for argument in contract_call.basic.arg_buffer.raw_arg_iter() {
        result.push('@');
        result.push_str(hex::encode(argument.to_boxed_bytes().as_slice()).as_str());
    }
    result
}