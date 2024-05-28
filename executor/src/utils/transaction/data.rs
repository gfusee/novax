use std::convert::From;
use std::str::FromStr;

use multiversx_sc::api::CallTypeApi;
use multiversx_sc::imports::{Tx, TxData, TxDataFunctionCall, TxEnv, TxFrom, TxGas, TxPayment, TxResultHandler, TxTo};
use multiversx_sc::types::{ContractCallWithEgld, ManagedAddress};
use multiversx_sc_scenario::scenario_model::{ScCallStep, ScDeployStep, TxCall, TxDeploy, TypedScCall, TypedScDeploy};
use multiversx_sdk::data::address::Address;
use num_bigint::BigUint;
use serde::{de, Deserialize, Serialize, Serializer};

/// Represents a transaction that is ready to be sent to the blockchain.
///
/// This structure contains the necessary information for the frontend to send a transaction
/// once the user has their wallet connected.
#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub struct SendableTransaction {
    /// The receiver's address as a string.
    pub receiver: String,
    /// The amount of EGLD to be sent along with the transaction.
    #[serde(serialize_with = "biguint_serialize")]
    #[serde(deserialize_with = "biguint_deserialize")]
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
    fn to_sendable_transaction(self) -> SendableTransaction;
}

// Implementations of `SendableTransactionConvertible` for various types, enabling them to be converted into `SendableTransaction`s.

impl SendableTransactionConvertible for SendableTransaction {
    fn to_sendable_transaction(self) -> SendableTransaction {
        self
    }
}

impl<Env, From, Payment, Gas, Data, RH> SendableTransactionConvertible for Tx<Env, From, ManagedAddress<Env::Api>, Payment, Gas, Data, RH>
    where
        Env: TxEnv,
        From: TxFrom<Env>,
        Payment: TxPayment<Env> + Clone, // TODO: evil clone
        Gas: TxGas<Env>,
        Data: TxDataFunctionCall<Env>,
        RH: TxResultHandler<Env>,
{
    fn to_sendable_transaction(self) -> SendableTransaction {
        let receiver = Address::from_bytes(self.to.to_byte_array()).to_bech32_string().unwrap();
        let egld_value = self.payment
            .clone() // TODO: evil clone
            .into_full_payment_data(&self.env).egld
            .map(|v| BigUint::from_bytes_be(v.value.to_bytes_be().as_slice()))
            .unwrap_or_default();

        SendableTransaction {
            receiver,
            egld_value,
            gas_limit: self.gas.gas_value(&self.env),
            data: self.to_call_data_string().to_string(),
        }
    }
}

fn biguint_serialize<S>(value: &BigUint, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer
{
    serializer.serialize_str(&value.to_string())
}

fn biguint_deserialize<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: de::Deserializer<'de>
{
    let string = String::deserialize(deserializer)?;

    Ok(BigUint::from_str(&string).unwrap())
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;

    use crate::SendableTransaction;

    #[test]
    fn test_serialize_sendable_transaction() {
        let tx = SendableTransaction {
            receiver: "erd1qqqqqqqqqqqqqpgq74myhunu4sfdpmskm6s6ul8k4cetjvhhlfpsaa20la".to_string(),
            egld_value: BigUint::from(10u8).pow(18),
            gas_limit: 600000000,
            data: "@6f6b@000000080de0b6b3a7640000000000081bc16d674ec80000".to_string(),
        };

        let result = serde_json::to_string(&tx).unwrap();
        let expected = r#"{"receiver":"erd1qqqqqqqqqqqqqpgq74myhunu4sfdpmskm6s6ul8k4cetjvhhlfpsaa20la","egld_value":"1000000000000000000","gas_limit":600000000,"data":"@6f6b@000000080de0b6b3a7640000000000081bc16d674ec80000"}"#;

        assert_eq!(result, expected);
    }

    #[test]
    fn test_deserialize_sendable_transaction() {
        let json = r#"{"receiver":"erd1qqqqqqqqqqqqqpgq74myhunu4sfdpmskm6s6ul8k4cetjvhhlfpsaa20la","egld_value":"1000000000000000000","gas_limit":600000000,"data":"@6f6b@000000080de0b6b3a7640000000000081bc16d674ec80000"}"#;

        let result: SendableTransaction = serde_json::from_str(json).unwrap();
        let expected = SendableTransaction {
            receiver: "erd1qqqqqqqqqqqqqpgq74myhunu4sfdpmskm6s6ul8k4cetjvhhlfpsaa20la".to_string(),
            egld_value: BigUint::from(10u8).pow(18),
            gas_limit: 600000000,
            data: "@6f6b@000000080de0b6b3a7640000000000081bc16d674ec80000".to_string(),
        };

        assert_eq!(result, expected);
    }
}